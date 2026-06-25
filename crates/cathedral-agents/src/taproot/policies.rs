pub trait Policy {
    fn can_create_asset(&self, name: &str, supply: u64) -> bool;
    fn can_transfer(&self, asset_id: &[u8], amount: u64, destination: &str) -> bool;
}
