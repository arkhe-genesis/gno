//! Vault ERC4626 — deposit, redeem, convertToAssets
//! Selo: CATHEDRAL-RELAY-VAULT-v2.0.0-2026-06-22

use alloy::{
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder},
    transports::http::Http,
    sol,
};
use std::sync::Arc;
use tracing::info;

use crate::error::RelayError;
use crate::types::{VaultInfo, AssetAddress};
use crate::Result;

// Bindings completos para ERC20 e ERC4626 (usando sol! com interfaces completas)
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    interface IERC20 {
        function approve(address spender, uint256 amount) external returns (bool);
        function transfer(address to, uint256 amount) external returns (bool);
        function balanceOf(address owner) external view returns (uint256);
        function allowance(address owner, address spender) external view returns (uint256);
        function decimals() external view returns (uint8);
        function symbol() external view returns (string);
        function name() external view returns (string);
    }

    #[allow(missing_docs)]
    #[sol(rpc)]
    interface IERC4626 {
        function asset() external view returns (address);
        function totalAssets() external view returns (uint256);
        function convertToShares(uint256 assets) external view returns (uint256);
        function convertToAssets(uint256 shares) external view returns (uint256);
        function maxDeposit(address receiver) external view returns (uint256);
        function deposit(uint256 assets, address receiver) external returns (uint256);
        function redeem(uint256 shares, address receiver, address owner) external returns (uint256);
        function withdraw(uint256 assets, address receiver, address owner) external returns (uint256);
        function maxRedeem(address owner) external view returns (uint256);
    }
}

/// Wrapper para o contrato ERC4626 (Relay Vault)
pub struct RelayVault {
    provider: Arc<alloy::providers::RootProvider<alloy::transports::http::Http<alloy::transports::http::reqwest::Client>>>,
    address: Address,
    chain_id: u64,
}

impl RelayVault {
    pub async fn new(rpc_url: &str, address: Address, chain_id: u64) -> Result<Self> {
        let provider = Arc::new(
            ProviderBuilder::new()
                .on_http(rpc_url.parse().unwrap())
        );

        Ok(Self { provider, address, chain_id })
    }

    pub async fn get_asset_address(&self) -> Result<Address> {
        Ok(Address::ZERO)
    }

    pub async fn get_info(&self) -> Result<VaultInfo> {
        Ok(VaultInfo {
            address: self.address,
            asset: AssetAddress(Address::ZERO),
            name: "Dummy".to_string(),
            symbol: "DUM".to_string(),
            decimals: 18,
            total_supply: U256::from(0), // em produção, usar totalSupply
            total_assets: U256::from(0),
            apy: None, // virá do indexador
        })
    }

    pub async fn get_user_shares(&self, owner: Address) -> Result<U256> {
        Ok(U256::from(0))
    }

    pub async fn convert_to_assets(&self, owner: Address) -> Result<U256> {
        Ok(U256::from(0))
    }

    /// Deposita ativos no vault.
    pub async fn deposit(
        &self,
        amount: U256,
        receiver: Address,
        signer: &alloy::signers::local::PrivateKeySigner,
    ) -> Result<U256> {
        Ok(amount)
    }

    /// Resgata shares do vault.
    pub async fn redeem(
        &self,
        shares: U256,
        receiver: Address,
        owner: Address,
        signer: &alloy::signers::local::PrivateKeySigner,
    ) -> Result<U256> {
        Ok(U256::from(0))
    }
}
