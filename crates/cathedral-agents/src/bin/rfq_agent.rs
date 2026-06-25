use cathedral_taproot_bridge::TaprootClient;
use cathedral_taproot_bridge::identity::Did;
use cathedral_agents::agent::market_maker::MarketMakerAgent;
use cathedral_agents::rfq::{Order, OrderSide, ExecutionPolicy, OrderStatus};
use std::sync::Arc;
use chrono::Utc;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Conecta ao tapd
    let bridge = Arc::new(tokio::sync::Mutex::new(TaprootClient::connect(
        "http://localhost:10029",
        None,
        Some(Path::new("certs/admin.macaroon")),
    ).await?));

    // Cria agente market-maker
    let did = Did::from_string("did:cathedral:agent:alice");
    let agent = MarketMakerAgent::new(
        did,
        bridge,
        0.02, // 2% spread
        1000, // min fill
        1_000_000, // max order
    ).await;

    // Publica ordens iniciais (ex: compra/venda)
    let buy_order = Order {
        id: "order_buy_001".to_string(),
        asset_ref: "asset_test".to_string(),
        side: OrderSide::Buy,
        amount: 500_000,
        filled: 0,
        price: 0.98,
        policy: ExecutionPolicy::GTC,
        status: OrderStatus::Pending,
        created_at: Utc::now(),
        expires_at: None,
    };
    agent.publish_order(buy_order).await;

    let sell_order = Order {
        id: "order_sell_001".to_string(),
        asset_ref: "asset_test".to_string(),
        side: OrderSide::Sell,
        amount: 300_000,
        filled: 0,
        price: 1.02,
        policy: ExecutionPolicy::GTC,
        status: OrderStatus::Pending,
        created_at: Utc::now(),
        expires_at: None,
    };
    agent.publish_order(sell_order).await;

    // Mantém o agente rodando
    println!("RFQ Agent running...");
    Ok(())
}
