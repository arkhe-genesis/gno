use crate::architect::CathedralArchitect;
use crate::BroadcastNotifier;
use std::sync::Arc;
use cathedral_wormgraph::{ImprovementProposal, WormGraphClient};

pub struct SelfImprovementOrchestrator {
    architect: CathedralArchitect,
    _wormgraph: Arc<WormGraphClient>,
    notifier: Arc<BroadcastNotifier>,
}

impl SelfImprovementOrchestrator {
    pub async fn run_cycle(&self) -> Result<Vec<ImprovementProposal>, String> {
        let analysis = self.architect.analyze_monorepo().await?;
        let proposals = self.architect.generate_proposals(&analysis).await?;
        let mut approved = Vec::new();
        for mut proposal in proposals {
            proposal.validation_status = cathedral_wormgraph::ValidationStatus::Approved;
            self.notifier.broadcast(proposal.clone()).await;
            approved.push(proposal);
        }
        Ok(approved)
    }
}
