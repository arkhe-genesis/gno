use std::sync::Arc;
use std::collections::HashMap;
use std::str::FromStr;
use ethers::types::{Address, U256};

use arkhe_kernel::substrato_4004::deps::{Action, EthicalFilter, EventStore, CrossChainEmitterV2, HybridZkVerifier, BatchSettlementEngine, X402XrplBridge, EscrowManager, B20Payment};
use arkhe_kernel::substrato_4004::b20_mapper::B20TokenMapper;
use arkhe_kernel::substrato_4004::policy_adapter::PolicyRegistryClient;
use arkhe_kernel::substrato_4004::compliance_engine::{ComplianceEngine, EthicalCompliance, PolicyCompliance};
use arkhe_kernel::substrato_4004::memo_tracer::MemoTracer;
use arkhe_kernel::substrato_4004::settlement_engine::B20SettlementEngine;
use arkhe_kernel::substrato_4004::cross_chain_bridge::B20XrplBridge;

async fn setup_compliance_engine() -> ComplianceEngine {
    let ethical_filter = Arc::new(EthicalFilter);
    let policy_registry = Arc::new(PolicyRegistryClient);
    let event_store = Arc::new(EventStore::new());

    let b20_mapper = Arc::new(B20TokenMapper {
        ethical_filter: ethical_filter.clone(),
        policy_registry: policy_registry.clone(),
    });

    ComplianceEngine {
        ethical_filter,
        policy_registry,
        b20_mapper,
        event_store,
    }
}

async fn setup_b20_xrpl_bridge() -> B20XrplBridge {
    let compliance_engine = Arc::new(setup_compliance_engine().await);
    let event_store = compliance_engine.event_store.clone();
    let cross_chain_emitter = Arc::new(CrossChainEmitterV2);

    let b20_settlement = Arc::new(B20SettlementEngine {
        b20_mapper: compliance_engine.b20_mapper.clone(),
        compliance_engine: compliance_engine.clone(),
        batch_engine: Arc::new(BatchSettlementEngine),
        cross_chain_emitter: cross_chain_emitter.clone(),
        zk_prover: Arc::new(HybridZkVerifier),
    });

    let xrpl_bridge = Arc::new(X402XrplBridge {
        escrow_manager: EscrowManager,
    });

    let memo_tracer = Arc::new(MemoTracer {
        event_store: event_store.clone(),
        cross_chain_emitter: cross_chain_emitter.clone(),
    });

    B20XrplBridge {
        b20_settlement,
        xrpl_bridge,
        cross_chain_emitter,
        memo_tracer,
    }
}

#[tokio::test]
async fn test_b20_compliance_full_flow() {
    let engine = setup_compliance_engine().await;

    let mut metadata = HashMap::new();
    metadata.insert("affects_human_dignity".to_string(), "false".to_string());
    metadata.insert("auditable".to_string(), "true".to_string());

    let action = Action {
        id: "b20-payment-1".to_string(),
        action_type: "payment_b20".to_string(),
        payload: serde_json::json!({
            "token": "0x0000000000000000000000000000000000000001",
            "from": "0x0000000000000000000000000000000000000002",
            "to": "0x0000000000000000000000000000000000000003",
            "amount": "1000000000000000000",
        }),
        metadata,
    };

    let verdict = engine.evaluate_compliance(&action).await.unwrap();
    assert!(verdict.overall);
    assert!(matches!(verdict.ethical, EthicalCompliance::Passed));
    assert!(matches!(verdict.policy, PolicyCompliance::Passed));
}

#[tokio::test]
async fn test_b20_freeze_and_seize() {
    let engine = setup_compliance_engine().await;

    let mut metadata = HashMap::new();
    metadata.insert("has_kill_switch".to_string(), "true".to_string());
    metadata.insert("respects_constitution".to_string(), "true".to_string());

    let action = Action {
        id: "freeze-1".to_string(),
        action_type: "freeze_and_seize".to_string(),
        payload: serde_json::json!({
            "token": "0x0000000000000000000000000000000000000001",
            "target": "0x0000000000000000000000000000000000000002", // Would fail normally, since the mock authorizes it
            "amount": "1000000",
        }),
        metadata,
    };

    // Note: Since mock `is_authorized` returns `true` (authorized),
    // and freeze_and_seize checks `if is_authorized { return Err(NotBlocked) }`, this will return Err.
    let result = engine.evaluate_compliance(&action).await;
    assert!(result.is_err(), "Expected failure because the target is mocked as not blocked");
}

#[tokio::test]
async fn test_b20_xrpl_bridge() {
    let bridge = setup_b20_xrpl_bridge().await;

    let payment = B20Payment {
        id: "payment-1".to_string(),
        token: Address::from_str("0x0000000000000000000000000000000000000001").unwrap(),
        from: Address::from_str("0x0000000000000000000000000000000000000002").unwrap(),
        to: Address::from_str("0x0000000000000000000000000000000000000003").unwrap(),
        amount: U256::from(1000000000000000000u64),
        memo: None,
    };

    let escrow_id = bridge.b20_to_xrpl_escrow(&payment).await.unwrap();
    assert!(!escrow_id.is_empty());

    let release_tx = bridge.xrpl_to_b20_release(&escrow_id, payment.to).await.unwrap();
    assert!(!release_tx.is_empty());
}
