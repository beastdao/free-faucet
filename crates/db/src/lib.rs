use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle, UserKey, UserValue};
use format_bytes::format_bytes;
use shared_types::{LogEntry, LogValue};
struct DbLogEntry(LogEntry);
pub struct SerializableLogValue<'a>(pub &'a LogValue);

pub struct DBMeta {
    pub journal_disk_space: u64,
    pub partition_count: usize,
    pub partition_size_limit: u64,
    pub log_entries: usize,
    pub log_disk_space: u64,
    pub log_segments: usize,
    pub claim_entries: usize,
    pub claim_disk_space: u64,
    pub claim_segments: usize,
}

#[derive(Clone)]
pub struct DB {
    partition_registry: PartitionHandle,
    partition_logs: PartitionHandle,
    keyspace: Keyspace,
    size_limit: u64,
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

trait DBErrorContext<T> {
    fn db_error_with_context(self, context: &'static str) -> Result<T, DBErrors>;
}

impl<T> DBErrorContext<T> for Result<T, fjall::Error> {
    fn db_error_with_context(self, context: &'static str) -> Result<T, DBErrors> {
        self.map_err(|e| DBErrors::DBError { context, source: e })
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
    pub fn new(path: &str, limit: u64) -> Result<Self, DBErrors> {
        let keyspace = Config::new(path)
            .max_write_buffer_size(1_024 * 1_024)
            .open()
            .map_err(|e| DBErrors::DBError {
                context: "init",
                source: e,
            })?;

        let registry = keyspace
            .open_partition(
                "claim",
                PartitionCreateOptions::default().compaction_strategy(
                    fjall::compaction::Strategy::Fifo(fjall::compaction::Fifo::new(limit, None)),
                ),
            )
            .db_error_with_context("init claim partition")?;

        let logs = keyspace
            .open_partition(
                "logs",
                PartitionCreateOptions::default().compaction_strategy(
                    fjall::compaction::Strategy::Fifo(fjall::compaction::Fifo::new(limit, None)),
                ),
            )
            .db_error_with_context("init logs partition")?;

        Ok(Self {
            partition_registry: registry,
            partition_logs: logs,
            keyspace: keyspace,
            size_limit: limit,
        })
    }

    pub fn insert_k_v_claim(&self, key: &str, value: u64) -> Result<(), DBErrors> {
        self.partition_registry
            .insert(key, value.to_be_bytes())
            .db_error_with_context("insert")
    }

    pub fn get_value_claim(&self, key: &str) -> Result<Option<u64>, DBErrors> {
        match self
            .partition_registry
            .get(key)
            .db_error_with_context("get")?
        {
            Some(v) => Ok(Some(convert_slice_to_u64(v))),
            None => Ok(None),
        }
    }

    // return 0 if no previous claims
    // reverse since we need largest timestamp
    // finds first element that satisfies "true for status"
    // find returns option -> transpose to work with result

    pub fn get_last_claim_timestamp(
        &self,
        range_low: u64,
        range_high: u64,
    ) -> Result<u64, DBErrors> {
        self.partition_logs
            .range(range_low.to_be_bytes()..(range_high + 1).to_be_bytes())
            .rev()
            .map(|res| {
                let (k, _) = res.db_error_with_context("get log keys from range")?;
                convert_slice_to_tuple(&k)
            })
            .find(|res| matches!(res, Ok((_, s)) if *s == 1))
            .transpose()
            .map(|opt| opt.map(|(ts, _)| ts).unwrap_or(0))
    }

    pub fn get_db_meta(&self) -> Result<DBMeta, DBErrors> {
        Ok(DBMeta {
            journal_disk_space: self.keyspace.disk_space(),
            partition_count: self.keyspace.partition_count(),
            partition_size_limit: self.size_limit,
            log_entries: self
                .partition_logs
                .len()
                .db_error_with_context("get log entries")?,
            log_disk_space: self.partition_logs.disk_space(),
            log_segments: self.partition_logs.segment_count(),
            claim_entries: self
                .partition_registry
                .len()
                .db_error_with_context("get claim entries")?,
            claim_disk_space: self.partition_registry.disk_space(),
            claim_segments: self.partition_registry.segment_count(),
        })
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
            .db_error_with_context("insert log")
    }

    pub fn get_value_log(&self, key: (u64, u8)) -> Result<Option<LogValue>, DBErrors> {
        let timestamp_bytes = key.0.to_be_bytes();
        let status_bytes = key.1.to_be_bytes();
        let key = format_bytes!(b"{}\0{}", timestamp_bytes, status_bytes);
        match self
            .partition_logs
            .get(&key)
            .db_error_with_context("get log value")?
        {
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
