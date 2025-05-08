use fjall::{Config, PartitionCreateOptions, PartitionHandle, UserKey, UserValue};
use format_bytes::format_bytes;
use shared_types::{LogEntry, LogValue};
struct DbLogEntry(LogEntry);
pub struct SerializableLogValue<'a>(pub &'a LogValue);

#[derive(Clone)]
pub struct DB {
    partition_registry: PartitionHandle,
    partition_logs: PartitionHandle,
}

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for LogValue {
    fn to_bytes(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).expect("should serialize")
    }
}

impl TryFrom<(UserKey, UserValue)> for DbLogEntry {
    type Error = DBErrors;
    fn try_from((key, value): (UserKey, UserValue)) -> Result<Self, Self::Error> {
        let key_tuple = convert_slice_to_tuple(&key)?;
        let log_value: LogValue = rmp_serde::from_slice(&value)
            .map_err(|e| DBErrors::ConversionError(format!("Deserialization error: {}", e)))?;

        Ok(DbLogEntry(LogEntry {
            key: key_tuple,
            value: log_value,
        }))
    }
}

//update in conveninece layer style.
#[derive(Debug, thiserror::Error)]
pub enum DBErrors {
    #[error("Database error during {context}: {source}")]
    DBError {
        context: &'static str,
        #[source]
        source: fjall::Error,
    },
    #[error("Conversion error: {0}")]
    ConversionError(String),
}

fn convert_slice_to_u64(slice: fjall::Slice) -> u64 {
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&slice);
    u64::from_be_bytes(bytes)
}

fn convert_slice_to_tuple(slice: &[u8]) -> Result<(u64, u8), DBErrors> {
    if slice.len() != 10 || slice[8] != 0 {
        return Err(DBErrors::ConversionError(
            "Key conversion error: Invalid slice format".to_string(),
        ));
    }

    let mut timestamp_bytes = [0u8; 8];
    timestamp_bytes.copy_from_slice(&slice[..8]);
    let timestamp = u64::from_be_bytes(timestamp_bytes);

    let status_byte = slice[9];

    Ok((timestamp, status_byte))
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
        let logs = keyspace
            .open_partition("logs", PartitionCreateOptions::default())
            .map_err(|e| DBErrors::DBError {
                context: "init",
                source: e,
            })?;

        Ok(Self {
            partition_registry: registry,
            partition_logs: logs,
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

    pub fn insert_k_v_logs(
        &self,
        timestamp: u64,
        status: bool,
        input: String,
        result: String,
    ) -> Result<(), DBErrors> {
        let timestamp_bytes = timestamp.to_be_bytes();
        let status_bytes = (status as u8).to_be_bytes();
        let key = format_bytes!(b"{}\0{}", timestamp_bytes, status_bytes);
        let log_struct = LogValue { input, result };
        let serialized = log_struct.to_bytes();
        self.partition_logs
            .insert(key, serialized)
            .map_err(|e| DBErrors::DBError {
                context: "insert",
                source: e,
            })
    }

    pub fn get_value_log(&self, key: (u64, u8)) -> Result<Option<LogValue>, DBErrors> {
        let timestamp_bytes = key.0.to_be_bytes();
        let status_bytes = (key.1 as u8).to_be_bytes();
        let key = format_bytes!(b"{}\0{}", timestamp_bytes, status_bytes);
        match self
            .partition_logs
            .get(&key)
            .map_err(|e| DBErrors::DBError {
                context: "get",
                source: e,
            })? {
            Some(v) => Ok(Some(rmp_serde::from_slice(&v).expect("should deserialize"))),
            None => Ok(None),
        }
    }

    pub fn iter_logs(&self) -> impl Iterator<Item = Result<LogEntry, DBErrors>> + '_ {
        self.partition_logs.iter().map(|item| {
            {
                item.map_err(|e| DBErrors::DBError {
                    context: "iter_logs",
                    source: e,
                })
            }
            .and_then(|res| Ok(DbLogEntry::try_from(res)?.0))
        })
    }
}
