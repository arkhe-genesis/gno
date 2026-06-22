pub struct NostrReplicator {}

impl NostrReplicator {
    pub fn default_relays(&self) -> &[String] {
        &[]
    }
    pub async fn publish_to_relays(
        &self,
        event: &nostr_sdk::Event,
        relays: &[String],
    ) -> Result<PublishedEvent, String> {
        Ok(PublishedEvent {
            event_id_hex: event.id.to_hex(),
            relay_urls: relays.to_vec(),
        })
    }
}

pub struct PublishedEvent {
    pub event_id_hex: String,
    pub relay_urls: Vec<String>,
}
