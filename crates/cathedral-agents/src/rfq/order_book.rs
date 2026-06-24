use tokio::sync::Mutex;
use chrono::Utc;
use super::{Order, OrderSide, OrderStatus};

pub struct OrderBook {
    pub orders: Mutex<Vec<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            orders: Mutex::new(Vec::new()),
        }
    }

    /// Adiciona uma nova ordem
    pub async fn add_order(&self, order: Order) {
        let mut orders = self.orders.lock().await;
        orders.push(order);
    }

    /// Encontra ordens de compra para um ativo específico
    pub async fn get_buy_orders(&self, asset_ref: &str) -> Vec<Order> {
        let orders = self.orders.lock().await;
        orders.iter()
            .filter(|o| o.asset_ref == asset_ref && o.side == OrderSide::Buy && o.status == OrderStatus::Pending)
            .cloned()
            .collect()
    }

    /// Encontra ordens de venda para um ativo específico
    pub async fn get_sell_orders(&self, asset_ref: &str) -> Vec<Order> {
        let orders = self.orders.lock().await;
        orders.iter()
            .filter(|o| o.asset_ref == asset_ref && o.side == OrderSide::Sell && o.status == OrderStatus::Pending)
            .cloned()
            .collect()
    }

    /// Remove ordens expiradas
    pub async fn clean_expired(&self) {
        let mut orders = self.orders.lock().await;
        let now = Utc::now();
        orders.retain(|o| {
            if let Some(expires) = o.expires_at {
                if now > expires {
                    return false; // remove
                }
            }
            true
        });
    }

    /// Encontra ordens com preço melhor ou igual ao limite
    pub async fn find_matching_orders(
        &self,
        asset_ref: &str,
        side: OrderSide,
        price_limit: f64,
        _amount: u64,
    ) -> Vec<Order> {
        let orders = self.orders.lock().await;
        let mut matching: Vec<Order> = orders.iter()
            .filter(|o| {
                o.asset_ref == asset_ref
                    && o.side == side
                    && o.status == OrderStatus::Pending
                    && match side {
                        OrderSide::Buy => o.price <= price_limit, // para compras, preço <= limite
                        OrderSide::Sell => o.price >= price_limit, // para vendas, preço >= limite
                    }
            })
            .cloned()
            .collect();
        // Ordena por preço (menor para compra, maior para venda)
        match side {
            OrderSide::Buy => matching.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap()),
            OrderSide::Sell => matching.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap()),
        }
        matching
    }
}
