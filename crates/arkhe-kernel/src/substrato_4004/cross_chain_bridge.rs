use std::sync::Arc;
use ethers::types::Address;
use sha2::Digest;

use crate::substrato_4004::deps::{B20Payment, X402XrplBridge, CrossChainEmitterV2, OrchestratorEvent};
use crate::substrato_4004::settlement_engine::{B20SettlementEngine, SettlementError};
use crate::substrato_4004::b20_mapper::{B20Operation, PolicyScope};
use crate::substrato_4004::memo_tracer::MemoTracer;
use crate::substrato_4004::compliance_engine::ComplianceError;

#[derive(Debug, Clone)]
pub enum BridgeError {
    ComplianceFailed(crate::substrato_4004::compliance_engine::ComplianceVerdict),
    Compliance(ComplianceError),
    Settlement(SettlementError),
    XrplBridge(String),
    EscrowNotReleased(String),
    CrossChain(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::ComplianceFailed(v) => write!(f, "Compliance check failed: {:?}", v),
            BridgeError::Compliance(e) => write!(f, "Compliance error: {}", e),
            BridgeError::Settlement(e) => write!(f, "Settlement error: {}", e),
            BridgeError::XrplBridge(s) => write!(f, "XRPL bridge error: {}", s),
            BridgeError::EscrowNotReleased(s) => write!(f, "Escrow not released: {}", s),
            BridgeError::CrossChain(s) => write!(f, "Cross chain error: {}", s),
        }
    }
}

impl From<ComplianceError> for BridgeError {
    fn from(e: ComplianceError) -> Self {
        BridgeError::Compliance(e)
    }
}

impl From<SettlementError> for BridgeError {
    fn from(e: SettlementError) -> Self {
        BridgeError::Settlement(e)
    }
}

pub struct B20XrplBridge {
    pub b20_settlement: Arc<B20SettlementEngine>,
    pub xrpl_bridge: Arc<X402XrplBridge>,
    pub cross_chain_emitter: Arc<CrossChainEmitterV2>,
    pub memo_tracer: Arc<MemoTracer>,
}

impl B20XrplBridge {
    pub async fn b20_to_xrpl_escrow(
        &self,
        payment: &B20Payment,
    ) -> Result<String, BridgeError> {
        let action = payment.to_action();
        let compliance = self.b20_settlement.compliance_engine.evaluate_compliance(&action).await?;

        if !compliance.overall {
            return Err(BridgeError::ComplianceFailed(compliance));
        }

        let escrow_address = self.get_bridge_escrow_address().await;
        let freeze_tx = self.b20_settlement.execute_b20_operation(&B20Operation::Transfer {
            token: payment.token,
            from: payment.from,
            to: escrow_address,
            amount: payment.amount,
            memo: Some(self.memo_tracer.generate_memo(&action)),
            policy_scope: PolicyScope::TransferSender,
        }).await?;

        let xrpl_escrow_id = self.xrpl_bridge.create_settlement_escrow(
            &payment.to_x402_payment()
        ).await.map_err(BridgeError::XrplBridge)?;

        self.cross_chain_emitter.emit_cross_chain(OrchestratorEvent::B20ToXrplBridge {
            b20_tx_hash: freeze_tx,
            xrpl_escrow_id: xrpl_escrow_id.clone(),
            amount: payment.amount.to_string(),
            token: format!("{:?}", payment.token),
            timestamp: chrono::Utc::now().timestamp(),
        }).await.map_err(BridgeError::CrossChain)?;

        Ok(xrpl_escrow_id)
    }

    pub async fn xrpl_to_b20_release(
        &self,
        xrpl_escrow_id: &str,
        b20_recipient: Address,
    ) -> Result<String, BridgeError> {
        let escrow_state = self.xrpl_bridge.escrow_manager.get_state(xrpl_escrow_id)
            .await.map_err(BridgeError::XrplBridge)?;

        if !escrow_state.released {
            return Err(BridgeError::EscrowNotReleased(xrpl_escrow_id.to_string()));
        }

        let escrow_address = self.get_bridge_escrow_address().await;
        let mut hasher = sha2::Sha256::new();
        sha2::Digest::update(&mut hasher, b"xrpl-release");
        sha2::Digest::update(&mut hasher, xrpl_escrow_id.as_bytes());
        let result = sha2::Digest::finalize(hasher);
        let mut memo = [0u8; 32];
        memo.copy_from_slice(&result);

        let release_tx = self.b20_settlement.execute_b20_operation(&B20Operation::Transfer {
            token: escrow_state.token,
            from: escrow_address,
            to: b20_recipient,
            amount: escrow_state.amount,
            memo: Some(memo),
            policy_scope: PolicyScope::TransferSender,
        }).await?;

        self.cross_chain_emitter.emit_cross_chain(OrchestratorEvent::XrplToB20Release {
            xrpl_escrow_id: xrpl_escrow_id.to_string(),
            b20_tx_hash: release_tx.clone(),
            recipient: format!("{:?}", b20_recipient),
            timestamp: chrono::Utc::now().timestamp(),
        }).await.map_err(BridgeError::CrossChain)?;

        Ok(release_tx)
    }

    async fn get_bridge_escrow_address(&self) -> Address {
        // Stub: In reality this would fetch a configured contract address
        Address::random()
    }
}
