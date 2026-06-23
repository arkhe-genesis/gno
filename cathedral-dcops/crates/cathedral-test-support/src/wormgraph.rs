use cathedral_wormgraph::{WormGraphBackend, LedgerEntry, ImprovementProposal, ProposalFilter, MemoryFilter, Result, WormGraphError};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct TestWormGraph {
    entries: DashMap<String, LedgerEntry>,
    proposals: DashMap<String, ImprovementProposal>,
    next_id: AtomicU64,
}

impl TestWormGraph {
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
            proposals: DashMap::new(),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn insert_proposal_sync(&self, mut proposal: ImprovementProposal) -> Result<()> {
        if proposal.id.is_empty() {
            proposal.id = format!("prop_{}", self.next_id.fetch_add(1, Ordering::SeqCst));
        }
        self.proposals.insert(proposal.id.clone(), proposal);
        Ok(())
    }

    pub fn populate_with_proposals(&self, count: usize, author: &str) -> Result<()> {
        for i in 0..count {
            let mut prop = ImprovementProposal::new(format!("Title {}", i), format!("Desc {}", i));
            prop.author_did = author.to_string();
            self.insert_proposal_sync(prop)?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl WormGraphBackend for TestWormGraph {
    async fn list_memories(&self, filter: MemoryFilter) -> Result<Vec<LedgerEntry>> {
        let mut vec: Vec<_> = self.entries.iter().map(|kv| kv.value().clone()).collect();
        // Aplica filtros e paginação
        if let Some(agent) = filter.agent_id {
            vec.retain(|e| e.agent_id == agent);
        }
        vec.sort_by_key(|e| -e.timestamp);
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);
        Ok(vec.into_iter().skip(offset).take(limit).collect())
    }

    async fn list_proposals(&self, filter: ProposalFilter) -> Result<Vec<ImprovementProposal>> {
        let mut vec: Vec<_> = self.proposals.iter().map(|kv| kv.value().clone()).collect();
        if let Some(risk) = filter.risk_level { vec.retain(|p| p.risk_level == risk); }
        vec.sort_by_key(|p| -p.created_at.timestamp());
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);
        Ok(vec.into_iter().skip(offset).take(limit).collect())
    }
}
