use axum::{Router, routing::get, response::Json};
use cathedral_scheduler::{HybridScheduler, WorkerRegistry, WorkerProfile, WorkerTier, TaskType, TeeType};
use cathedral_agi::{AGICore, OllamaClient};
use cathedral_episodic::EpisodicSync;
use cathedral_fallback::FallbackChain;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tracing::{info, warn, error};

// Healthcheck endpoint
async fn healthcheck() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": "3.0.0",
        "uptime": chrono::Utc::now().timestamp(),
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("🏛️ Cathedral Edge Agent v3.0.0 (Hybrid Mode) starting...");

    // 1. Inicializar componentes
    let registry = Arc::new(WorkerRegistry::new());
    let scheduler = Arc::new(HybridScheduler::new(
        registry.clone(),
        15000,
        0.7,
        100_000,
    ));
    let mut fallback = FallbackChain::new(8000);

    // 2. Registrar workers
    register_workers(&registry, &mut fallback).await?;
    let fallback = Arc::new(fallback);

    // 3. Inicializar memória episódica
    let episodic_path = std::path::PathBuf::from("data/episodic.jsonl");
    std::fs::create_dir_all("data")?;

    // SQLite uses "sqlite:" or "sqlite://" format
    let db_url = format!("sqlite://{}", episodic_path.to_str().unwrap());

    // Need to create the db file if it doesn't exist to avoid sqlx connection errors
    if !episodic_path.exists() {
        std::fs::File::create(&episodic_path)?;
    }

    let episodic = Arc::new(EpisodicSync::new("edge-agent".to_string(), &db_url).await?);

    // 4. Inicializar LLM client (Ollama)
    let llm = Arc::new(OllamaClient::new("llama3.1:8b"));
    info!("🔍 Checking Ollama health...");
    if !llm.healthcheck().await {
        warn!("⚠️ Ollama not reachable at http://localhost:11434. AGI Core will use simulation fallback.");
    } else {
        info!("✅ Ollama is reachable");
    }

    // 5. Inicializar AGI Core
    let agi_core = Arc::new(tokio::sync::Mutex::new(
        AGICore::new(llm.clone(), Some(episodic.clone()))
    ));

    // 6. Healthcheck server (axum)
    let app = Router::new()
        .route("/health", get(healthcheck))
        .route("/metrics", get(|| async move {
            // let stats = scheduler.stats().await;
            Json(serde_json::json!({
                "workers": "stats_mock",
                "agi_confidence": { "confidence": 0.5 }, // placeholder
            }))
        }));

    let server_handle = tokio::spawn(async {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:9898").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    info!("🌐 Healthcheck server running on port 9898");

    // 7. Loop principal de processamento
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    let mut task_counter = 0;

    let scheduler_clone = scheduler.clone();

    loop {
        tokio::select! {
            _ = interval.tick() => {
                task_counter += 1;
                let task = format!("Process task #{}", task_counter);

                // Decisão do scheduler
                let decision = scheduler_clone.schedule(TaskType::Inference).await;
                info!("📋 Task {} scheduled to {}", task_counter, decision.selected_worker);

                // Tentar executar via fallback
                let start = std::time::Instant::now();
                match fallback.execute(&task, TaskType::Inference).await {
                    Ok(result) => {
                        scheduler_clone.record_result(&decision.selected_worker, true).await?;
                        info!("✅ Task completed in {:?}: {}", start.elapsed(), result);
                    }
                    Err(e) => {
                        scheduler_clone.record_result(&decision.selected_worker, false).await?;
                        warn!("❌ Task failed: {}", e);
                    }
                }

                // Processar com AGI Core a cada 3 tasks
                if task_counter % 3 == 0 {
                    info!("🧠 Processing with AGI Core...");
                    let mut core = agi_core.lock().await;
                    let prompt = format!("What should I do about task {}? Summary: {}", task_counter, task);
                    match core.process(&prompt).await {
                        Ok(response) => {
                            info!("🤖 AGI Core response: {}", &response[..200.min(response.len())]);
                        }
                        Err(e) => {
                            error!("AGI Core error: {}", e);
                        }
                    }
                }

                // Estatísticas periódicas
                if task_counter % 10 == 0 {
                    let stats = scheduler_clone.stats().await;
                    let meta = agi_core.lock().await.meta_state().clone();
                    info!("📊 Stats: workers={}, active={}, confidence={:.2}",
                        stats.total_workers, stats.active_workers, meta.confidence);
                }
            }
            _ = signal::ctrl_c() => {
                info!("🛑 Shutting down gracefully...");
                break;
            }
        }
    }

    server_handle.abort();
    info!("👋 Cathedral Edge Agent stopped");

    Ok(())
}

async fn register_workers(registry: &Arc<WorkerRegistry>, fallback: &mut FallbackChain) -> anyhow::Result<()> {
    use cathedral_fallback::{WorkerExecutor, WorkerTier as FallbackWorkerTier};

    // Datacenter worker
    let local = WorkerProfile {
        worker_id: "edge-datacenter".to_string(),
        tier: WorkerTier::Datacenter,
        endpoint: "http://localhost:9898".to_string(),
        cost_per_hour: 0.005,
        latency_p50_ms: 50,
        latency_p95_ms: 100,
        reputation: 1.0,
        stake_sats: 0,
        last_attestation: chrono::Utc::now().timestamp(),
        tasks_completed: 0,
        tasks_failed: 0,
        available: true,
        tee_type: TeeType::None,
        capabilities: vec!["cpu".to_string(), "cuda".to_string()],
    };
    registry.register(local).await?;

    fallback.add_worker(WorkerExecutor {
        id: "edge-datacenter".to_string(),
        tier: FallbackWorkerTier::Datacenter,
        endpoint: "http://localhost:9898".to_string(),
        tee_attested: true,
    });


    // DePIN workers (simulados)
    for i in 1..4 {
        let worker_id = format!("depin-worker-{}", i);
        let endpoint = format!("http://depin-{}.io:8080", i);

        let worker = WorkerProfile {
            worker_id: worker_id.clone(),
            tier: WorkerTier::DePinGpu,
            endpoint: endpoint.clone(),
            cost_per_hour: 0.08 + (i as f64 * 0.02),
            latency_p50_ms: 150 + (i as u64 * 50),
            latency_p95_ms: 400 + (i as u64 * 100),
            reputation: 0.7 + (i as f32 * 0.08),
            stake_sats: 100_000 + (i as u64 * 20_000),
            last_attestation: chrono::Utc::now().timestamp(),
            tasks_completed: 0,
            tasks_failed: 0,
            available: true,
            tee_type: TeeType::IoNet,
            capabilities: vec!["cuda".to_string()],
        };
        registry.register(worker).await?;

        fallback.add_worker(WorkerExecutor {
            id: worker_id,
            tier: FallbackWorkerTier::DePinGpu,
            endpoint,
            tee_attested: true,
        });
    }

    info!("✅ Registered {} workers", 4);
    Ok(())
}
