use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LogValue {
    pub input: String,
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub key: (u64, u8),
    pub value: LogValue,
}
