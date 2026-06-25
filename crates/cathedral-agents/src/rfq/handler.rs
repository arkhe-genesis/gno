use cathedral_taproot_bridge::TaprootClient;
use crate::rfq::{RfqRequest, RfqResponse, Order, ExecutionPolicy, OrderStatus, ExecutionReport, OrderSide};
use crate::rfq::pricing::PricingPolicy;
use crate::rfq::order_book::OrderBook;
use std::sync::Arc;
use chrono::{Utc, Duration};
use cathedral_wormgraph::{WormGraphClient, MemoryEntry};

pub struct RfqHandler {
    bridge: Arc<tokio::sync::Mutex<TaprootClient>>,
    pricing_engine: Arc<dyn PricingPolicy>,
    pub order_book: Arc<OrderBook>,
    wormgraph: Arc<WormGraphClient>,
    min_fill_amount: u64,
    max_order_size: u64,
}

impl RfqHandler {
    pub fn new(
        bridge: Arc<tokio::sync::Mutex<TaprootClient>>,
        pricing_engine: Arc<dyn PricingPolicy>,
        order_book: Arc<OrderBook>,
        wormgraph: Arc<WormGraphClient>,
        min_fill_amount: u64,
        max_order_size: u64,
    ) -> Self {
        Self {
            bridge,
            pricing_engine,
            order_book,
            wormgraph,
            min_fill_amount,
            max_order_size,
        }
    }

    /// Processa uma RFQ recebida
    pub async fn handle_rfq(&self, request: RfqRequest) -> Result<RfqResponse, Box<dyn std::error::Error>> {
        // Verifica se o ativo é suportado
        // (validação de asset_ref via bridge)
        let asset_id_bytes = hex::decode(request.asset_ref.replace("group_key_", "")).unwrap_or_default();
        let mut client = self.bridge.lock().await;
        let asset_info = client.query_universe(asset_id_bytes, None).await?;
        if asset_info.asset_leaves.is_empty() { // Using asset_leaves as it looks like universe_response structure might differ
            return Err("Asset not found in Universe".into());
        }

        // Verifica limites de quantidade
        if request.amount < self.min_fill_amount {
            return Err("Amount below minimum fill".into());
        }
        if request.amount > self.max_order_size {
            return Err("Amount exceeds max order size".into());
        }

        // Calcula preço
        let price = self.pricing_engine.price(
            &request.asset_ref,
            request.amount,
            request.side.clone(),
        );

        // Verifica se o preço solicitado é aceitável (se fornecido)
        if let Some(requested_price) = request.requested_price {
            match request.side {
                OrderSide::Buy => {
                    if price > requested_price {
                        return Err("Price above requested limit".into());
                    }
                }
                OrderSide::Sell => {
                    if price < requested_price {
                        return Err("Price below requested limit".into());
                    }
                }
            }
        }

        // Cria a resposta
        let quote_id = format!("quote_{}", uuid::Uuid::new_v4());
        let response = RfqResponse {
            request_id: request.id.clone(),
            price,
            max_fill: request.amount,
            expiry: Utc::now() + Duration::minutes(1),
            quote_id: quote_id.clone(),
        };

        // Registra no WormGraph
        let content = serde_json::to_string(&serde_json::json!({
            "event": "rfq_response",
            "request_id": request.id,
            "quote_id": quote_id,
            "asset_ref": request.asset_ref,
            "price": price,
            "amount": request.amount,
            "side": request.side,
            "peer_did": request.peer_did,
        })).unwrap_or_default();

        let _ = self.wormgraph.append_memory(
            &request.peer_did,
            MemoryEntry { content }
        ).await;

        Ok(response)
    }

    /// Executa uma ordem baseada em uma RFQ aceita
    pub async fn execute_order(
        &self,
        request: RfqRequest,
        response: RfqResponse,
        policy: ExecutionPolicy,
    ) -> Result<ExecutionReport, Box<dyn std::error::Error>> {
        let order_id = format!("order_{}", uuid::Uuid::new_v4());

        // Cria a ordem
        let order = Order {
            id: order_id.clone(),
            asset_ref: request.asset_ref.clone(),
            side: request.side.clone(),
            amount: request.amount,
            filled: 0,
            price: response.price,
            policy: policy.clone(),
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            expires_at: Some(response.expiry),
        };

        // Adiciona ao order book
        self.order_book.add_order(order.clone()).await;

        // Simula matching (na prática, isso seria mais complexo)
        let executed = match policy {
            ExecutionPolicy::FOK => self.execute_fok(order).await,
            ExecutionPolicy::IOC => self.execute_ioc(order).await,
            ExecutionPolicy::GTC => self.execute_gtc(order).await,
        }?;

        // Se executado, usa a bridge para transferir ativos
        if executed.status == OrderStatus::Filled || executed.status == OrderStatus::PartiallyFilled {
            self.execute_on_chain(&executed).await?;
        }

        // Registra no WormGraph
        let content = serde_json::to_string(&serde_json::json!({
            "event": "order_execution",
            "order_id": order_id,
            "asset_ref": request.asset_ref,
            "filled_amount": executed.filled_amount,
            "price": response.price,
            "status": executed.status,
        })).unwrap_or_default();

        let _ = self.wormgraph.append_memory(
            &request.peer_did,
            MemoryEntry { content }
        ).await;

        Ok(executed)
    }

    async fn execute_fok(&self, order: Order) -> Result<ExecutionReport, Box<dyn std::error::Error>> {
        // Verifica se há liquidez suficiente para preencher completamente
        let available = self.check_liquidity(&order.asset_ref, order.amount).await?;
        if available < order.amount {
            return Ok(ExecutionReport {
                order_id: order.id,
                filled_amount: 0,
                price: order.price,
                status: OrderStatus::Cancelled,
                tx_id: None,
                timestamp: Utc::now(),
            });
        }
        // Preenche completamente
        Ok(ExecutionReport {
            order_id: order.id,
            filled_amount: order.amount,
            price: order.price,
            status: OrderStatus::Filled,
            tx_id: None,
            timestamp: Utc::now(),
        })
    }

    async fn execute_ioc(&self, order: Order) -> Result<ExecutionReport, Box<dyn std::error::Error>> {
        let available = self.check_liquidity(&order.asset_ref, order.amount).await?;
        let filled = order.amount.min(available);
        let status = if filled == order.amount {
            OrderStatus::Filled
        } else if filled > 0 {
            OrderStatus::PartiallyFilled
        } else {
            OrderStatus::Cancelled
        };
        Ok(ExecutionReport {
            order_id: order.id,
            filled_amount: filled,
            price: order.price,
            status,
            tx_id: None,
            timestamp: Utc::now(),
        })
    }

    async fn execute_gtc(&self, order: Order) -> Result<ExecutionReport, Box<dyn std::error::Error>> {
        // Mantém a ordem ativa até ser preenchida ou cancelada
        // (simplificado: preenche o máximo possível agora)
        self.execute_ioc(order).await
    }

    async fn check_liquidity(&self, _asset_ref: &str, amount: u64) -> Result<u64, Box<dyn std::error::Error>> {
        // Consulta saldo do ativo via bridge
        // (implementação depende da API do tapd para balance)
        // Por enquanto, retorna amount (assume liquidez total)
        Ok(amount)
    }

    async fn execute_on_chain(&self, _report: &ExecutionReport) -> Result<(), Box<dyn std::error::Error>> {
        // Usa a bridge para enviar ativos
        // (precisa de detalhes do destinatário, pubkey, etc.)
        // Exemplo:
        // self.bridge.send_asset(&order.asset_ref, report.filled_amount, dest_pubkey, None).await?;
        Ok(())
    }
}
