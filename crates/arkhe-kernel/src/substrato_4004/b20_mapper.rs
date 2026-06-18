use ethers::types::{Address, U256, Bytes};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::substrato_4004::deps::{Action, EthicalFilter, FilterVerdict};
use crate::substrato_4004::policy_adapter::{PolicyRegistryClient, PolicyError};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum B20Operation {
    Transfer {
        token: Address,
        from: Address,
        to: Address,
        amount: U256,
        memo: Option<[u8; 32]>,
        policy_scope: PolicyScope,
    },
    Mint {
        token: Address,
        to: Address,
        amount: U256,
        memo: Option<[u8; 32]>,
    },
    Burn {
        token: Address,
        from: Address,
        amount: U256,
        memo: Option<[u8; 32]>,
        burn_type: BurnType,
    },
    UpdatePolicy {
        token: Address,
        scope: PolicyScope,
        policy_id: u64,
    },
    Pause {
        token: Address,
        features: Vec<PausableFeature>,
        pause: bool,
    },
    UpdateMultiplier {
        token: Address,
        new_multiplier: U256, // WAD precision
    },
    Announce {
        token: Address,
        internal_calls: Vec<Bytes>,
        id: u64,
        description: String,
        uri: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyScope {
    TransferSender,
    TransferReceiver,
    TransferExecutor,
    MintReceiver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BurnType {
    Caller,      // burn próprio
    Blocked,     // burnBlocked (freeze-and-seize)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PausableFeature {
    Transfer,
    Mint,
    Burn,
}

#[derive(Debug, Clone)]
pub enum MapperError {
    EthicalViolation(Vec<crate::substrato_4004::deps::LayerViolation>),
    PolicyDenied(String),
    SupplyCapExceeded,
    NotBlocked(Address),
    UnsupportedActionType(String),
    MissingField(String),
    ParseError(String),
    PolicyError(PolicyError),
}

impl From<PolicyError> for MapperError {
    fn from(e: PolicyError) -> Self {
        MapperError::PolicyError(e)
    }
}

pub struct B20TokenMapper {
    pub ethical_filter: Arc<EthicalFilter>,
    pub policy_registry: Arc<PolicyRegistryClient>,
}

fn extract_address(action: &Action, field: &str) -> Result<Address, MapperError> {
    let s = action.payload.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| MapperError::MissingField(field.to_string()))?;
    s.parse::<Address>().map_err(|e| MapperError::ParseError(e.to_string()))
}

fn extract_u256(action: &Action, field: &str) -> Result<U256, MapperError> {
    let s = action.payload.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| MapperError::MissingField(field.to_string()))?;
    U256::from_dec_str(s).map_err(|e| MapperError::ParseError(e.to_string()))
}

fn extract_optional_memo(action: &Action) -> Result<Option<[u8; 32]>, MapperError> {
    if let Some(memo_val) = action.payload.get("memo") {
        if let Some(s) = memo_val.as_str() {
            let decoded = hex::decode(s).map_err(|e| MapperError::ParseError(e.to_string()))?;
            if decoded.len() == 32 {
                let mut memo = [0u8; 32];
                memo.copy_from_slice(&decoded);
                return Ok(Some(memo));
            } else {
                return Err(MapperError::ParseError("Memo must be 32 bytes".to_string()));
            }
        }
    }
    Ok(None)
}

fn hash_memo(prefix: &str, action: &Action) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(action.id.as_bytes());
    let result = hasher.finalize();
    let mut memo = [0u8; 32];
    memo.copy_from_slice(&result);
    memo
}

fn extract_policy_scope(action: &Action) -> Result<PolicyScope, MapperError> {
    let s = action.payload.get("scope")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MapperError::MissingField("scope".to_string()))?;
    match s {
        "TransferSender" => Ok(PolicyScope::TransferSender),
        "TransferReceiver" => Ok(PolicyScope::TransferReceiver),
        "TransferExecutor" => Ok(PolicyScope::TransferExecutor),
        "MintReceiver" => Ok(PolicyScope::MintReceiver),
        _ => Err(MapperError::ParseError("Invalid policy scope".to_string())),
    }
}

fn extract_u64(action: &Action, field: &str) -> Result<u64, MapperError> {
    let s = action.payload.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| MapperError::MissingField(field.to_string()))?;
    s.parse::<u64>().map_err(|e| MapperError::ParseError(e.to_string()))
}

fn extract_pausable_features(action: &Action) -> Result<Vec<PausableFeature>, MapperError> {
    let arr = action.payload.get("features")
        .and_then(|v| v.as_array())
        .ok_or_else(|| MapperError::MissingField("features".to_string()))?;
    let mut features = Vec::new();
    for v in arr {
        let s = v.as_str().ok_or_else(|| MapperError::ParseError("Feature not a string".to_string()))?;
        match s {
            "Transfer" => features.push(PausableFeature::Transfer),
            "Mint" => features.push(PausableFeature::Mint),
            "Burn" => features.push(PausableFeature::Burn),
            _ => return Err(MapperError::ParseError(format!("Invalid feature: {}", s))),
        }
    }
    Ok(features)
}

impl B20TokenMapper {
    pub async fn map_action(&self, action: &Action) -> Result<B20Operation, MapperError> {
        match self.ethical_filter.evaluate(action).await {
            FilterVerdict::Passed => {}
            FilterVerdict::Failed(v) => return Err(MapperError::EthicalViolation(v)),
        }

        match action.action_type.as_str() {
            "payment_b20" => {
                let token = extract_address(action, "token")?;
                let from = extract_address(action, "from")?;
                let to = extract_address(action, "to")?;
                let amount = extract_u256(action, "amount")?;
                let memo = extract_optional_memo(action)?;

                let sender_policy = self.policy_registry
                    .get_policy(token, PolicyScope::TransferSender)
                    .await?;

                if !self.policy_registry.is_authorized(sender_policy, from).await? {
                    return Err(MapperError::PolicyDenied("sender".to_string()));
                }

                Ok(B20Operation::Transfer {
                    token,
                    from,
                    to,
                    amount,
                    memo,
                    policy_scope: PolicyScope::TransferSender,
                })
            }
            "mint_b20" => {
                let token = extract_address(action, "token")?;
                let to = extract_address(action, "to")?;
                let amount = extract_u256(action, "amount")?;
                let memo = extract_optional_memo(action)?;

                let current_supply = self.get_total_supply(token).await?;
                let cap = self.get_supply_cap(token).await?;

                if current_supply + amount > cap {
                    return Err(MapperError::SupplyCapExceeded);
                }

                Ok(B20Operation::Mint { token, to, amount, memo })
            }
            "freeze_and_seize" => {
                let token = extract_address(action, "token")?;
                let target = extract_address(action, "target")?;
                let amount = extract_u256(action, "amount")?;

                let sender_policy = self.policy_registry
                    .get_policy(token, PolicyScope::TransferSender)
                    .await?;

                if self.policy_registry.is_authorized(sender_policy, target).await? {
                    return Err(MapperError::NotBlocked(target));
                }

                Ok(B20Operation::Burn {
                    token,
                    from: target,
                    amount,
                    memo: Some(hash_memo("freeze-and-seize", action)),
                    burn_type: BurnType::Blocked,
                })
            }
            "update_policy" => {
                let token = extract_address(action, "token")?;
                let scope = extract_policy_scope(action)?;
                let policy_id = extract_u64(action, "policy_id")?;

                Ok(B20Operation::UpdatePolicy { token, scope, policy_id })
            }
            "pause_b20" => {
                let token = extract_address(action, "token")?;
                let features = extract_pausable_features(action)?;

                Ok(B20Operation::Pause { token, features, pause: true })
            }
            "unpause_b20" => {
                let token = extract_address(action, "token")?;
                let features = extract_pausable_features(action)?;

                Ok(B20Operation::Pause { token, features, pause: false })
            }
            "update_multiplier" => {
                let token = extract_address(action, "token")?;
                let multiplier = extract_u256(action, "multiplier")?;

                Ok(B20Operation::UpdateMultiplier { token, new_multiplier: multiplier })
            }
            _ => Err(MapperError::UnsupportedActionType(action.action_type.clone())),
        }
    }

    async fn get_total_supply(&self, _token: Address) -> Result<U256, MapperError> {
        Ok(U256::zero()) // Stub implementation
    }

    async fn get_supply_cap(&self, _token: Address) -> Result<U256, MapperError> {
        Ok(U256::MAX) // Stub implementation
    }
}
