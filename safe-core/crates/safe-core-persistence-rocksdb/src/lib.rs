//! Safe-Core Persistence — Backend RocksDB
//!
//! # Alternativas Consideradas
//! | Backend | Vantagens | Desvantagens |
//! |---------|-----------|--------------|
//! | RocksDB | Performance, LSM-tree, compressão | FFI C++, build lento (~5min) |
//! | sled | Rust puro, rápido de build | Menos maduro, API instável |
//! | SQLite | SQL, maduro | Overhead para KV simples |
//! | redb | Rust puro, B-tree | Novo, pouca adoção |
//!
//! # Escolha: RocksDB
//! - Melhor performance para workloads write-heavy (audit logs)
//! - Suporte a column families (separar audit, config, state)
//! - Compressão LZ4/Zstd integrada
//! - Backup e snapshot nativos

use rocksdb::{DB, ColumnFamilyDescriptor, Options, WriteBatch};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::path::Path;

/// Erros de persistência
#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("RocksDB error: {0}")]
    RocksDb(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

/// Column families do Safe-Core
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnFamily {
    Audit,
    Config,
    State,
    Merkle,
    Consensus,
}

impl ColumnFamily {
    pub fn as_str(&self) -> &str {
        match self {
            ColumnFamily::Audit => "audit",
            ColumnFamily::Config => "config",
            ColumnFamily::State => "state",
            ColumnFamily::Merkle => "merkle",
            ColumnFamily::Consensus => "consensus",
        }
    }
}

/// Backend de persistência RocksDB
pub struct RocksDbBackend {
    db: DB,
}

impl RocksDbBackend {
    /// Abre ou cria o banco de dados.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, PersistenceError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        // Configurações de performance
        opts.set_max_open_files(1000);
        opts.set_use_fsync(false);
        opts.set_bytes_per_sync(8388608);
        opts.optimize_for_point_lookup(1024);

        let cf_names = vec!["audit", "config", "state", "merkle", "consensus"];
        let cf_descriptors: Vec<_> = cf_names.iter()
            .map(|name| ColumnFamilyDescriptor::new(*name, Options::default()))
            .collect();

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)
            .map_err(|e| PersistenceError::RocksDb(e.to_string()))?;

        Ok(Self { db })
    }

    /// Insere um valor serializado.
    pub fn put<T: Serialize>(
        &self,
        cf: ColumnFamily,
        key: &[u8],
        value: &T,
    ) -> Result<(), PersistenceError> {
        let serialized = serde_json::to_vec(value)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        let cf_handle = self.db.cf_handle(cf.as_str())
            .ok_or_else(|| PersistenceError::RocksDb(format!("Column family {} not found", cf.as_str())))?;

        self.db.put_cf(&cf_handle, key, &serialized)
            .map_err(|e| PersistenceError::RocksDb(e.to_string()))
    }

    /// Recupera um valor.
    pub fn get<T: for<'de> Deserialize<'de>>(
        &self,
        cf: ColumnFamily,
        key: &[u8],
    ) -> Result<T, PersistenceError> {
        let cf_handle = self.db.cf_handle(cf.as_str())
            .ok_or_else(|| PersistenceError::RocksDb(format!("Column family {} not found", cf.as_str())))?;

        let data = self.db.get_cf(&cf_handle, key)
            .map_err(|e| PersistenceError::RocksDb(e.to_string()))?
            .ok_or_else(|| PersistenceError::KeyNotFound(hex::encode(key)))?;

        serde_json::from_slice(&data)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))
    }

    /// Verifica se uma chave existe.
    pub fn exists(&self, cf: ColumnFamily, key: &[u8]) -> Result<bool, PersistenceError> {
        let cf_handle = self.db.cf_handle(cf.as_str())
            .ok_or_else(|| PersistenceError::RocksDb(format!("Column family {} not found", cf.as_str())))?;

        self.db.get_cf(&cf_handle, key)
            .map_err(|e| PersistenceError::RocksDb(e.to_string()))
            .map(|v| v.is_some())
    }

    /// Deleta uma chave.
    pub fn delete(&self, cf: ColumnFamily, key: &[u8]) -> Result<(), PersistenceError> {
        let cf_handle = self.db.cf_handle(cf.as_str())
            .ok_or_else(|| PersistenceError::RocksDb(format!("Column family {} not found", cf.as_str())))?;

        self.db.delete_cf(&cf_handle, key)
            .map_err(|e| PersistenceError::RocksDb(e.to_string()))
    }

    /// Batch write atômico.
    pub fn batch_write(&self, operations: Vec<BatchOp>) -> Result<(), PersistenceError> {
        let mut batch = WriteBatch::default();

        for op in operations {
            let cf_handle = self.db.cf_handle(op.cf.as_str())
                .ok_or_else(|| PersistenceError::RocksDb(format!("Column family {} not found", op.cf.as_str())))?;

            match op.op_type {
                BatchOpType::Put => {
                    batch.put_cf(&cf_handle, &op.key, &op.value);
                }
                BatchOpType::Delete => {
                    batch.delete_cf(&cf_handle, &op.key);
                }
            }
        }

        self.db.write(batch)
            .map_err(|e| PersistenceError::RocksDb(e.to_string()))
    }

    /// Itera sobre todas as entradas de uma column family.
    pub fn iter_cf(&self, cf: ColumnFamily) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PersistenceError> {
        let cf_handle = self.db.cf_handle(cf.as_str())
            .ok_or_else(|| PersistenceError::RocksDb(format!("Column family {} not found", cf.as_str())))?;

        let mut results = Vec::new();
        let iter = self.db.iterator_cf(&cf_handle, rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, value) = item.map_err(|e| PersistenceError::RocksDb(e.to_string()))?;
            results.push((key.to_vec(), value.to_vec()));
        }

        Ok(results)
    }

    /// Cria snapshot para backup consistente.
    pub fn snapshot(&self) -> Result<Vec<u8>, PersistenceError> {
        let checkpoint = rocksdb::checkpoint::Checkpoint::new(&self.db)
            .map_err(|e| PersistenceError::RocksDb(e.to_string()))?;

        let path = std::env::temp_dir().join(format!("safe-core-snapshot-{}", uuid::Uuid::new_v4()));
        checkpoint.create_checkpoint(&path)
            .map_err(|e| PersistenceError::RocksDb(e.to_string()))?;

        Ok(path.to_string_lossy().into_owned().into_bytes())
    }
}

/// Operação de batch
pub struct BatchOp {
    pub cf: ColumnFamily,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub op_type: BatchOpType,
}

pub enum BatchOpType {
    Put,
    Delete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_put_get() {
        let dir = TempDir::new().unwrap();
        let db = RocksDbBackend::open(dir.path()).unwrap();

        db.put(ColumnFamily::Config, b"key1", &"value1".to_string()).unwrap();
        let value: String = db.get(ColumnFamily::Config, b"key1").unwrap();
        assert_eq!(value, "value1");
    }

    #[test]
    fn test_batch_write() {
        let dir = TempDir::new().unwrap();
        let db = RocksDbBackend::open(dir.path()).unwrap();

        let ops = vec![
            BatchOp {
                cf: ColumnFamily::Audit,
                key: b"log1".to_vec(),
                value: b"entry1".to_vec(),
                op_type: BatchOpType::Put,
            },
            BatchOp {
                cf: ColumnFamily::Audit,
                key: b"log2".to_vec(),
                value: b"entry2".to_vec(),
                op_type: BatchOpType::Put,
            },
        ];

        db.batch_write(ops).unwrap();

        let entries = db.iter_cf(ColumnFamily::Audit).unwrap();
        assert_eq!(entries.len(), 2);
    }
}
