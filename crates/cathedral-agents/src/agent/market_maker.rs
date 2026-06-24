use cathedral_taproot_bridge::TaprootClient;
use crate::rfq::handler::RfqHandler;
use crate::rfq::pricing::MarketPricing;
use crate::rfq::order_book::OrderBook;
use crate::rfq::{RfqRequest, RfqResponse, Order, ExecutionPolicy, OrderStatus, ExecutionReport};
use cathedral_taproot_bridge::identity::Did;
use cathedral_wormgraph::WormGraphClient;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MarketMakerAgent {
    pub did: Did,
    pub bridge: Arc<tokio::sync::Mutex<TaprootClient>>,
    pub rfq_handler: Arc<RfqHandler>,
    pub active_orders: RwLock<Vec<Order>>,
}

impl MarketMakerAgent {
    pub async fn new(
        did: Did,
        bridge: Arc<tokio::sync::Mutex<TaprootClient>>,
        spread: f64,
        min_fill: u64,
        max_order: u64,
    ) -> Self {
        let order_book = Arc::new(OrderBook::new());
        let pricing = Arc::new(MarketPricing::new(spread));
        let wormgraph = Arc::new(WormGraphClient::new("http://localhost:8080"));
        let handler = Arc::new(RfqHandler::new(
            bridge.clone(),
            pricing,
            order_book.clone(),
            wormgraph,
            min_fill,
            max_order,
        ));

        Self {
            did,
            bridge,
            rfq_handler: handler,
            active_orders: RwLock::new(Vec::new()),
        }
    }

    /// Escuta por RFQs (via MCP ou gRPC) e responde
    pub async fn listen_for_rfqs(&self) {
        // Implementação de listener
        // Exemplo: via MCP Bridge, recebe mensagens com tipo "rfq"
        // Para cada mensagem, chama self.handle_rfq()
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            // Simula recebimento de RFQ
            // (na prática, isso seria um canal de comunicação)
        }
    }

    /// Publica uma ordem no order book
    pub async fn publish_order(&self, order: Order) {
        self.rfq_handler.order_book.add_order(order).await;
    }

    /// Executa uma ordem
    pub async fn execute_order(&self, request: RfqRequest, response: RfqResponse) -> Result<ExecutionReport, Box<dyn std::error::Error>> {
        self.rfq_handler.execute_order(request, response, ExecutionPolicy::IOC).await
    }

    /// Obtém status de uma ordem
    pub async fn get_order_status(&self, order_id: &str) -> Option<OrderStatus> {
        let orders = self.rfq_handler.order_book.orders.lock().await;
        orders.iter().find(|o| o.id == order_id).map(|o| o.status.clone())
    }
}
