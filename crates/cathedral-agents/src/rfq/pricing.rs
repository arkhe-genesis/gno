use std::collections::HashMap;
use tokio::sync::RwLock;
use super::OrderSide;

pub trait PricingPolicy: Send + Sync {
    fn price(&self, asset_ref: &str, amount: u64, side: OrderSide) -> f64;
}

/// Estratégia de preço baseada em mercado (cenário simples)
pub struct MarketPricing {
    base_price: RwLock<HashMap<String, f64>>,
    spread: f64, // spread percentual (ex: 0.01 = 1%)
}

impl MarketPricing {
    pub fn new(spread: f64) -> Self {
        Self {
            base_price: RwLock::new(HashMap::new()),
            spread,
        }
    }

    pub async fn update_price(&self, asset_ref: &str, price: f64) {
        let mut map = self.base_price.write().await;
        map.insert(asset_ref.to_string(), price);
    }
}

impl PricingPolicy for MarketPricing {
    fn price(&self, asset_ref: &str, _amount: u64, side: OrderSide) -> f64 {
        let base = self.base_price.blocking_read().get(asset_ref).cloned().unwrap_or(1.0);
        let adjusted = match side {
            OrderSide::Buy => base * (1.0 + self.spread),  // compra paga spread
            OrderSide::Sell => base * (1.0 - self.spread), // venda recebe menos
        };
        adjusted
    }
}

/// Estratégia de preço dinâmica (ex: com base em liquidez)
pub struct DynamicPricing {
    pub base_prices: RwLock<HashMap<String, f64>>,
    pub liquidity_pools: RwLock<HashMap<String, f64>>, // quantidade disponível
}

impl PricingPolicy for DynamicPricing {
    fn price(&self, asset_ref: &str, amount: u64, side: OrderSide) -> f64 {
        let base = self.base_prices.blocking_read().get(asset_ref).cloned().unwrap_or(1.0);
        let liquidity = self.liquidity_pools.blocking_read().get(asset_ref).cloned().unwrap_or(0.0);
        // Quanto maior a ordem em relação à liquidez, maior o spread
        let spread_factor = (amount as f64 / (liquidity + 1.0)).min(0.5);
        let spread = 0.01 + spread_factor * 0.05;
        match side {
            OrderSide::Buy => base * (1.0 + spread),
            OrderSide::Sell => base * (1.0 - spread),
        }
    }
}
