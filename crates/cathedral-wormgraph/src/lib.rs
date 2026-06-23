use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub id: String,
}

pub struct WormGraphClient {
    pub endpoint: String,
    pub storage_path: PathBuf,
}

impl WormGraphClient {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            storage_path: PathBuf::from("wormgraph_storage.json"),
        }
    }

    pub async fn append_memory(&self, _did: &str, entry: MemoryEntry) -> Result<(), ()> {
        let serialized = serde_json::to_string(&entry).map_err(|_| ())?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.storage_path)
            .map_err(|_| ())?;

        writeln!(file, "{}", serialized).map_err(|_| ())?;
        Ok(())
    }

    pub async fn get_memories(&self, _did: &str, limit: usize) -> Result<Vec<MemoryEntry>, ()> {
        if !self.storage_path.exists() {
            return Ok(vec![]);
        }

        let mut file = OpenOptions::new().read(true).open(&self.storage_path).map_err(|_| ())?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|_| ())?;

        let memories: Vec<MemoryEntry> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .take(limit)
            .collect();

        Ok(memories)
    }

    pub async fn search_similar(&self, _did: &str, query: &str, limit: usize) -> Result<Vec<MemoryEntry>, ()> {
        if !self.storage_path.exists() {
            return Ok(vec![]);
        }

        let mut file = OpenOptions::new().read(true).open(&self.storage_path).map_err(|_| ())?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|_| ())?;

        let memories: Vec<MemoryEntry> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .filter(|m: &MemoryEntry| m.content.contains(query))
            .take(limit)
            .collect();

        Ok(memories)
    }
}
