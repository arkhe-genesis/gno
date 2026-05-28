//! Cliente SPB
use crate::types::StrOrder;

pub struct SpbClient;

impl SpbClient {
    pub fn new() -> Self { Self }
    pub async fn send_order(&self, _order: &StrOrder) -> Result<String, String> {
        Ok("STR_CONFIRMED".into())
    }
}

pub struct SettlementEngine;
impl SettlementEngine {
    pub fn execute_dvp(&self) -> bool { true }
}
