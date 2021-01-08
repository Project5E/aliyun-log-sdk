mod proto;

use proto::{Content, Log};
use std::fmt;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

pub use proto::LogGroup;
use bytes::buf::UninitSlice;
use bytes::BytesMut;

/// The minimal unit representing a log record.
pub struct Record {
    time: u32,
    key_values: BTreeMap<String, String>,
}

impl Default for Record {
    fn default() -> Self {
        Self {
            time: chrono::Utc::now().timestamp() as u32,
            key_values: BTreeMap::new()
        }
    }
}

impl Deref for Record {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.key_values
    }
}

impl DerefMut for Record {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.key_values
    }
}

impl Record {
    pub fn new(time: u32) -> Self {
        Self { time, key_values: Default::default() }
    }
}

impl From<log::Record<'_>> for Record {
    fn from(log_record: log::Record<'_>) -> Self {
        let mut record = Record::default();
        record.insert("level".to_string(), log_record.level().to_string());
        record.insert("module".to_string(), log_record.module_path().unwrap_or("-").to_string());
        record.insert("file".to_string(), log_record.file().unwrap_or("-").to_string());
        record.insert("line".to_string(), log_record.line().map_or("-".to_string(), |l| l.to_string()));
        record.insert("target".to_string(), log_record.target().to_string());
        record.insert("message".to_string(), fmt::format(*log_record.args()));

        record
    }
}

impl From<Record> for Log {
    fn from(record: Record) -> Self {
        let Record { time, key_values } = record;
        Log::new(
            time,
            key_values
                .into_iter()
                .map(|(key, value)| Content::new(key, value))
                .collect(),
        )
    }
}
