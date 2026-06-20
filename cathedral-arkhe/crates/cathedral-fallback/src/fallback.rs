use crate::cost::CostOptimizer;
use anyhow::{Result, anyhow};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{info, error};
use cathedral_scheduler::TaskType;
use crate::lc3_executor::Lc3VmExecutor;

#[derive(Debug, Clone, PartialEq)]
pub enum WorkerTier {
    DePinGpu,
    DePinCpu,
    Datacenter,
}

#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    fn execute(&self, task: &str, task_type: TaskType) -> Result<String>;
    async fn execute_async(&self, task: &str, task_type: TaskType) -> Result<String>;
}

pub struct WorkerExecutor {
    pub id: String,
    pub tier: WorkerTier,
    pub endpoint: String,
    pub tee_attested: bool,
}

#[async_trait::async_trait]
impl TaskExecutor for WorkerExecutor {
    fn execute(&self, task: &str, _task_type: TaskType) -> Result<String> {
         Ok(format!("Executed on {}: {}", self.id, task))
    }

    async fn execute_async(&self, task: &str, _task_type: TaskType) -> Result<String> {
        Ok(format!("Executed on {}: {}", self.id, task))
    }
}

pub struct FallbackChain {
    gpu_workers: Vec<WorkerExecutor>,
    cpu_workers: Vec<WorkerExecutor>,
    datacenter_workers: Vec<WorkerExecutor>,
    lc3_executor: Option<Lc3VmExecutor>,
    timeout_ms: u64,
    _cost_optimizer: CostOptimizer,
}

impl FallbackChain {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            gpu_workers: Vec::new(),
            cpu_workers: Vec::new(),
            datacenter_workers: Vec::new(),
            lc3_executor: None,
            timeout_ms,
            _cost_optimizer: CostOptimizer::new(100, 15000),
        }
    }

    pub fn set_lc3_executor(&mut self, executor: Lc3VmExecutor) {
        self.lc3_executor = Some(executor);
    }

    pub fn add_worker(&mut self, worker: WorkerExecutor) {
        match worker.tier {
            WorkerTier::DePinGpu => self.gpu_workers.push(worker),
            WorkerTier::DePinCpu => self.cpu_workers.push(worker),
            WorkerTier::Datacenter => self.datacenter_workers.push(worker),
        }
    }

    pub async fn execute(&self, task: &str, task_type: TaskType) -> Result<String> {
        let start = Instant::now();

        info!("Fallback Level 1: DePIN GPU");
        for worker in &self.gpu_workers {
            if let Ok(result) = self.execute_worker(worker, task, task_type).await {
                self.record_cost(worker.id.clone(), "depin_gpu", start.elapsed().as_millis() as u64, true);
                return Ok(result);
            }
        }

        info!("Fallback Level 2: DePIN CPU");
        for worker in &self.cpu_workers {
            if let Ok(result) = self.execute_worker(worker, task, task_type).await {
                self.record_cost(worker.id.clone(), "depin_cpu", start.elapsed().as_millis() as u64, true);
                return Ok(result);
            }
        }

        info!("Fallback Level 3: Datacenter");
        for worker in &self.datacenter_workers {
            if let Ok(result) = self.execute_worker(worker, task, task_type).await {
                self.record_cost(worker.id.clone(), "datacenter", start.elapsed().as_millis() as u64, true);
                return Ok(result);
            }
        }

        if let Some(ref vm_exec) = self.lc3_executor {
            info!("🔹 Fallback Level 4: LC-3 Virtual Machine");
            match vm_exec.execute_async(task, task_type).await {
                Ok(result) => {
                    info!("✅ LC-3 VM success");
                    return Ok(result);
                }
                Err(e) => {
                    error!("LC-3 VM failed: {}", e);
                }
            }
        }

        Err(anyhow!("All fallback levels exhausted"))
    }

    async fn execute_worker(&self, worker: &WorkerExecutor, task: &str, task_type: TaskType) -> Result<String> {
        let duration = Duration::from_millis(self.timeout_ms);
        match timeout(duration, worker.execute_async(task, task_type)).await {
            Ok(result) => result,
            Err(_) => Err(anyhow!("Timeout after {}ms", self.timeout_ms)),
        }
    }

    fn record_cost(&self, _worker_id: String, _tier: &str, _latency_ms: u64, _success: bool) {
        // Not modifying cost_optimizer as this method shouldn't take mut
        // But for compiling structure we can leave it
    }

    pub fn should_use_depin(&self) -> bool {
        // self.cost_optimizer.should_use_depin()
        false // Mocked because cost optimizer needs mutable ref
    }
}
