use ethers::types::Address;
use crate::substrato_4004::b20_mapper::PolicyScope;

#[derive(Debug, Clone)]
pub struct PolicyError(pub String);

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct PolicyRegistryClient;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PolicyType {
    Blocklist = 0,
    Allowlist = 1,
}

impl PolicyRegistryClient {
    pub async fn create_policy(
        &self,
        _admin: Address,
        _policy_type: PolicyType,
        _initial_accounts: Vec<Address>,
    ) -> Result<u64, PolicyError> {
        Ok(1)
    }

    pub async fn is_authorized(&self, _policy_id: u64, _account: Address) -> Result<bool, PolicyError> {
        Ok(true)
    }

    pub async fn update_blocklist(
        &self,
        _policy_id: u64,
        _block: bool,
        _accounts: Vec<Address>,
    ) -> Result<(), PolicyError> {
        Ok(())
    }

    pub async fn get_policy(
        &self,
        _token: Address,
        _scope: PolicyScope,
    ) -> Result<u64, PolicyError> {
        Ok(1)
    }
}
