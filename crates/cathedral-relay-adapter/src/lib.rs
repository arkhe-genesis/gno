//! Cathedral-OS Relay Vaults Adapter (Unificado)
//! Selo: CATHEDRAL-RELAY-ADAPTER-v2.0.0-2026-06-22

pub mod client;
pub mod vault;
pub mod bridge;
pub mod types;
pub mod error;

pub use client::RelayIndexerClient;
pub use vault::RelayVault;
pub use bridge::RelayBridge;
pub use types::*;
pub use error::RelayError;

use alloy::primitives::{Address, U256};

pub type Result<T> = std::result::Result<T, RelayError>;

/// Adapter unificado para Relay Vaults.
/// Expõe API direta para depósito, resgate e bridge cross-chain.
pub struct RelayAdapter {
    pub vault: RelayVault,
    pub bridge: RelayBridge,
    pub indexer: RelayIndexerClient,
}

impl RelayAdapter {
    pub async fn new(
        rpc_url: &str,
        indexer_url: &str,
        vault_address: Address,
        bridge_address: Address,
        chain_id: u64,
    ) -> Result<Self> {
        let vault = RelayVault::new(rpc_url, vault_address, chain_id).await?;
        let bridge = RelayBridge::new(rpc_url, bridge_address, chain_id).await?;
        let indexer = RelayIndexerClient::new(indexer_url);

        Ok(Self { vault, bridge, indexer })
    }

    // ========================================================================
    // VAULT OPERATIONS (Yield)
    // ========================================================================

    pub async fn deposit(
        &self,
        amount: U256,
        signer: &alloy::signers::local::PrivateKeySigner,
    ) -> Result<U256> {
        let receiver = signer.address();
        self.vault.deposit(amount, receiver, signer).await
    }

    pub async fn redeem(
        &self,
        shares: U256,
        signer: &alloy::signers::local::PrivateKeySigner,
    ) -> Result<U256> {
        let owner = signer.address();
        let receiver = owner;
        self.vault.redeem(shares, receiver, owner, signer).await
    }

    pub async fn get_vault_balance(&self, user: Address) -> Result<BalanceInfo> {
        let asset = self.vault.get_asset_address().await?;
        let total_assets = self.vault.convert_to_assets(user).await?;
        let yield_earned = self.indexer.get_yield_earned(user, AssetAddress(asset)).await?;

        Ok(BalanceInfo {
            total_assets,
            yield_earned,
            pending_rewards: U256::from(0),
        })
    }

    // ========================================================================
    // BRIDGE OPERATIONS (Cross-Chain)
    // ========================================================================

    pub async fn bridge_cross_chain(
        &self,
        request: BridgeRequest,
        signer: &alloy::signers::local::PrivateKeySigner,
    ) -> Result<BridgeResponse> {
        // 1. Executa bridge (ECDSA)
        let response = self.bridge.bridge(request, signer).await?;

        Ok(response)
    }

    // ========================================================================
    // QUERY OPERATIONS (Indexer)
    // ========================================================================

    pub async fn list_vaults(&self) -> Result<Vec<VaultInfo>> {
        self.indexer.list_vaults().await
    }
}
