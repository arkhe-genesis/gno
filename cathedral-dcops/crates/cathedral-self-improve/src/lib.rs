pub mod architect;
pub mod orchestrator;

pub struct BroadcastNotifier;
impl BroadcastNotifier {
    pub async fn broadcast(&self, _proposal: cathedral_wormgraph::ImprovementProposal) {}
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<String> {
        let (_tx, rx) = tokio::sync::broadcast::channel(1);
        rx
    }
}
