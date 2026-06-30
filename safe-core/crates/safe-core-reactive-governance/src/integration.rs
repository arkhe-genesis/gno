//! Integration traits for UED Teacher and Sparse Router.

use crate::ReactiveLog;
use std::sync::Arc;

/// Trait for UED Teachers to query governance state.
#[async_trait::async_trait]
pub trait UedGovernance {
    /// Check if the system is currently frozen.
    async fn is_frozen(&self) -> bool;

    /// Get the reward adjustment for this teacher.
    async fn get_reward_adjustment(&self, teacher_id: &str) -> f64;

    /// Get the last rollback STH, if any.
    async fn get_rollback_sth(&self) -> Option<Vec<u8>>;
}

/// Trait for Sparse-Dense routers to query governance state.
#[async_trait::async_trait]
pub trait SparseRouterGovernance {
    /// Check if a specific routing path is banned.
    async fn is_route_banned(&self, router_id: &str, from_module: &str, to_module: &str) -> bool;

    /// Check if the system is frozen.
    async fn is_frozen(&self) -> bool;
}

/// Implement the traits for ReactiveLog.
#[async_trait::async_trait]
impl UedGovernance for ReactiveLog {
    async fn is_frozen(&self) -> bool {
        self.is_frozen().await
    }

    async fn get_reward_adjustment(&self, teacher_id: &str) -> f64 {
        self.get_teacher_reward_delta(teacher_id).await
    }

    async fn get_rollback_sth(&self) -> Option<Vec<u8>> {
        self.get_last_rollback_sth().await
    }
}

#[async_trait::async_trait]
impl SparseRouterGovernance for ReactiveLog {
    async fn is_route_banned(&self, router_id: &str, from_module: &str, to_module: &str) -> bool {
        self.is_route_banned(router_id, from_module, to_module).await
    }

    async fn is_frozen(&self) -> bool {
        self.is_frozen().await
    }
}
