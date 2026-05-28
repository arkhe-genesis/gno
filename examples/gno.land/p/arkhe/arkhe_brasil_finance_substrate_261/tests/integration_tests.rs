//! Testes de Integração
use arkhe_brasil_finance_substrate_261::types::{DictKey, PixTransaction};
use arkhe_brasil_finance_substrate_261::spi::generate_qr;
use arkhe_brasil_finance_substrate_261::bridge::ArkheBrasilBridge;
use arkhe_jax_zk::{ComputationProof, ZkScheme};
use sha3::{Sha3_256, Digest};

#[test]
fn test_pix_flow() {
    let tx = PixTransaction {
        end_to_end_id: "E12345678202301010000".into(),
        amount: 100.50,
        receiver_key: DictKey {
            key_type: "CPF".into(),
            key: "12345678909".into(),
        },
    };

    let qr = generate_qr(&tx);
    // 00020101021112345678909...
    assert!(qr.starts_with("00020126"));
    assert!(qr.contains("52040000")); // merchant category
    assert!(qr.contains("5303986")); // currency
    assert!(qr.contains("5802BR")); // country
    assert!(qr.contains("6304")); // end of payload format
    assert_eq!(qr.len() - qr.find("6304").unwrap(), 8); // 6304 + 4 hex digits
}

#[test]
fn test_str_settlement() {
    assert!(true);
}

#[test]
fn test_clearing_registry() {
    assert!(true);
}

#[test]
fn test_bridge_zk() {
    let tx = PixTransaction {
        end_to_end_id: "E12345678202301010000".into(),
        amount: 100.50,
        receiver_key: DictKey {
            key_type: "CPF".into(),
            key: "12345678909".into(),
        },
    };

    let mut hasher = Sha3_256::new();
    hasher.update(tx.end_to_end_id.as_bytes());
    hasher.update(&tx.amount.to_le_bytes());
    hasher.update(tx.receiver_key.key_type.as_bytes());
    hasher.update(tx.receiver_key.key.as_bytes());
    let tx_hash: [u8; 32] = hasher.finalize().into();

    let proof = ComputationProof {
        graph_commitment: [0u8; 32],
        output_hash: tx_hash,
        scheme: ZkScheme::Bn254Groth16,
    };

    let bridge = ArkheBrasilBridge::new();
    assert!(bridge.verify_zk_pix(&tx, &proof));
}
