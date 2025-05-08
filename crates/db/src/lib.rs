use fjall::{Config, PartitionCreateOptions, PartitionHandle};

#[derive(Clone)]
pub struct DB {
    partition_registry: PartitionHandle,
}

#[derive(Debug, thiserror::Error)]
pub enum DBErrors {
    #[error("Database error during {context}: {source}")]
    DBError {
        context: &'static str,
        #[source]
        source: fjall::Error,
    },
}

fn convert_slice_to_u64(slice: fjall::Slice) -> u64 {
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&slice);
    u64::from_be_bytes(bytes)
}

impl DB {
    pub fn new(path: &str, partition_name: &str) -> Result<Self, DBErrors> {
        let keyspace = Config::new(path).open().map_err(|e| DBErrors::DBError {
            context: "init",
            source: e,
        })?;
        let registry = keyspace
            .open_partition(partition_name, PartitionCreateOptions::default())
            .map_err(|e| DBErrors::DBError {
                context: "init",
                source: e,
            })?;
        Ok(Self {
            partition_registry: registry,
        })
    }

    pub fn insert_k_v_claim(&self, key: &str, value: u64) -> Result<(), DBErrors> {
        self.partition_registry
            .insert(key, value.to_be_bytes())
            .map_err(|e| DBErrors::DBError {
                context: "insert",
                source: e,
            })
    }

    pub fn get_value_claim(&self, key: &str) -> Result<Option<u64>, DBErrors> {
        match self
            .partition_registry
            .get(key)
            .map_err(|e| DBErrors::DBError {
                context: "get",
                source: e,
            })? {
            Some(v) => Ok(Some(convert_slice_to_u64(v))),
            None => Ok(None),
        }
    }
}
