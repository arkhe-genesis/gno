use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::substrato_4004::deps::{Action, EthicalFilter, EventStore, FilterVerdict, LayerViolation, OrchestratorEvent, B20Constants};
use crate::substrato_4004::policy_adapter::PolicyRegistryClient;
use crate::substrato_4004::b20_mapper::{B20TokenMapper, B20Operation, PolicyScope, PausableFeature, MapperError, BurnType};

pub struct ComplianceEngine {
    pub ethical_filter: Arc<EthicalFilter>,
    pub policy_registry: Arc<PolicyRegistryClient>,
    pub b20_mapper: Arc<B20TokenMapper>,
    pub event_store: Arc<EventStore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceVerdict {
    pub ethical: EthicalCompliance,
    pub policy: PolicyCompliance,
    pub pause: PauseCompliance,
    pub role: RoleCompliance,
    pub overall: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EthicalCompliance {
    Passed,
    Failed(Vec<LayerViolation>),
}

impl EthicalCompliance {
    pub fn is_passed(&self) -> bool {
        matches!(self, EthicalCompliance::Passed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCompliance {
    Passed,
    Denied(String),
}

impl PolicyCompliance {
    pub fn is_passed(&self) -> bool {
        matches!(self, PolicyCompliance::Passed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PauseCompliance {
    Passed,
    Paused(PausableFeature),
}

impl PauseCompliance {
    pub fn is_passed(&self) -> bool {
        matches!(self, PauseCompliance::Passed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleCompliance {
    Passed,
    MissingRole([u8; 32]),
}

impl RoleCompliance {
    pub fn is_passed(&self) -> bool {
        matches!(self, RoleCompliance::Passed)
    }
}

#[derive(Debug, Clone)]
pub enum ComplianceError {
    Mapping(MapperError),
    Event(String),
    Policy(crate::substrato_4004::policy_adapter::PolicyError),
}

impl std::fmt::Display for ComplianceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplianceError::Mapping(e) => write!(f, "Mapping error: {:?}", e),
            ComplianceError::Event(e) => write!(f, "Event error: {}", e),
            ComplianceError::Policy(e) => write!(f, "Policy error: {:?}", e),
        }
    }
}

impl From<crate::substrato_4004::policy_adapter::PolicyError> for ComplianceError {
    fn from(e: crate::substrato_4004::policy_adapter::PolicyError) -> Self {
        ComplianceError::Policy(e)
    }
}

impl ComplianceEngine {
    pub async fn evaluate_compliance(&self, action: &Action) -> Result<ComplianceVerdict, ComplianceError> {
        let ethical = match self.ethical_filter.evaluate(action).await {
            FilterVerdict::Passed => EthicalCompliance::Passed,
            FilterVerdict::Failed(v) => EthicalCompliance::Failed(v),
        };

        let b20_op = match self.b20_mapper.map_action(action).await {
            Ok(op) => op,
            Err(e) => return Err(ComplianceError::Mapping(e)),
        };

        let policy = self.check_policies(&b20_op).await?;
        let pause = self.check_pause_state(&b20_op).await?;
        let role = self.check_roles(&b20_op, action).await?;

        let verdict = ComplianceVerdict {
            ethical: ethical.clone(),
            policy: policy.clone(),
            pause: pause.clone(),
            role: role.clone(),
            overall: ethical.is_passed() && policy.is_passed() && pause.is_passed() && role.is_passed(),
        };

        self.event_store.emit(OrchestratorEvent::ComplianceChecked {
            action_id: action.id.clone(),
            verdict: verdict.clone(),
            timestamp: chrono::Utc::now().timestamp(),
        }).await.map_err(ComplianceError::Event)?;

        Ok(verdict)
    }

    async fn check_policies(&self, op: &B20Operation) -> Result<PolicyCompliance, ComplianceError> {
        match op {
            B20Operation::Transfer { token, from, to, .. } => {
                let sender_policy = self.policy_registry.get_policy(*token, PolicyScope::TransferSender).await?;
                let receiver_policy = self.policy_registry.get_policy(*token, PolicyScope::TransferReceiver).await?;

                let sender_ok = self.policy_registry.is_authorized(sender_policy, *from).await?;
                let receiver_ok = self.policy_registry.is_authorized(receiver_policy, *to).await?;

                if !sender_ok {
                    return Ok(PolicyCompliance::Denied(format!("sender {:?} blocked by policy {}", from, sender_policy)));
                }
                if !receiver_ok {
                    return Ok(PolicyCompliance::Denied(format!("receiver {:?} blocked by policy {}", to, receiver_policy)));
                }

                Ok(PolicyCompliance::Passed)
            }
            B20Operation::Mint { token, to, .. } => {
                let policy = self.policy_registry.get_policy(*token, PolicyScope::MintReceiver).await?;
                if !self.policy_registry.is_authorized(policy, *to).await? {
                    return Ok(PolicyCompliance::Denied(format!("mint receiver {:?} blocked", to)));
                }
                Ok(PolicyCompliance::Passed)
            }
            _ => Ok(PolicyCompliance::Passed),
        }
    }

    async fn check_pause_state(&self, _op: &B20Operation) -> Result<PauseCompliance, ComplianceError> {
        // Mock pause state check, normally calls IB20 `pausedFeatures`
        Ok(PauseCompliance::Passed)
    }

    async fn check_roles(&self, op: &B20Operation, action: &Action) -> Result<RoleCompliance, ComplianceError> {
        let required_role = match op {
            B20Operation::Mint { .. } => B20Constants::MINT_ROLE,
            B20Operation::Burn { burn_type: BurnType::Caller, .. } => B20Constants::BURN_ROLE,
            B20Operation::Burn { burn_type: BurnType::Blocked, .. } => B20Constants::BURN_BLOCKED_ROLE,
            B20Operation::Pause { pause: true, .. } => B20Constants::PAUSE_ROLE,
            B20Operation::Pause { pause: false, .. } => B20Constants::UNPAUSE_ROLE,
            B20Operation::UpdateMultiplier { .. } => B20Constants::OPERATOR_ROLE,
            _ => return Ok(RoleCompliance::Passed),
        };

        // Stub out role checking logic.
        // It would typically use IB20 hasRole function.
        let _caller = action.payload.get("caller").and_then(|v| v.as_str());

        Ok(RoleCompliance::Passed)
    }
}
