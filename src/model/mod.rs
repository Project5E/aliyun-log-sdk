mod proto;
mod implementation;

use std::borrow::Cow;

use bytes::Bytes;
use quick_protobuf::{MessageWrite, Writer};

use crate::error::Result;
use proto::Pair;
pub use proto::{Content, LogTag, Log, LogGroup};

impl<'a> Pair<'a> {
    pub fn new(key: &'a str, value: &'a str) -> Self {
        Self {
            key: Cow::Borrowed(key),
            value: Cow::Borrowed(value)
        }
    }

    pub fn into_owned(self) -> Pair<'static> {
        Pair {
            key: Cow::Owned(self.key.into_owned()),
            value: Cow::Owned(self.value.into_owned())
        }
    }
}

impl<'a> Log<'a> {
    pub fn new(time: u32) -> Self {
        Self {
            time,
            contents: vec![]
        }
    }

    pub fn into_owned(self) -> Log<'static> {
        Log {
            time: self.time,
            contents: self.contents.into_iter().map(|c| c.into_owned()).collect()
        }
    }

    pub fn time(&self) -> u32 {
        self.time
    }

    pub fn contents(&self) -> &[Content<'a>]{
        self.contents.as_slice()
    }

    pub fn push<'b>(&mut self, content: Content<'b>) -> &mut Self where 'b: 'a {
        self.contents.push(content);
        self
    }
}

impl Default for Log<'_> {
    fn default() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self::new(timestamp as u32)
    }
}

impl<'a> LogGroup<'a> {
    pub fn new(logs: Vec<Log<'a>>) -> Self {
        Self {
            logs,
            topic: None,
            source: None,
            log_tags: vec![]
        }
    }

    pub fn into_owned(self) -> LogGroup<'static> {
        let Self { logs, topic, source, log_tags } = self;
        LogGroup {
            logs: logs.into_iter().map(|l| l.into_owned()).collect(),
            topic: topic.map(|t| Cow::Owned(t.into_owned())),
            source: source.map(|s| Cow::Owned(s.into_owned())),
            log_tags: log_tags.into_iter().map(|l| l.into_owned()).collect()
        }
    }

    pub fn set_topic(&mut self, topic: &'a str) -> &mut Self {
        self.topic = Some(Cow::Borrowed(topic));
        self
    }

    pub fn set_source(&mut self, source: &'a str) -> &mut Self {
        self.source = Some(Cow::Borrowed(source));
        self
    }

    pub fn push_tag(&mut self, tag: LogTag<'a>) -> &mut Self {
        self.log_tags.push(tag);
        self
    }

    pub fn encode(&self) -> Result<Bytes> {
        let mut buf = Vec::with_capacity(self.get_size());
        let mut writer = Writer::new(&mut buf);
        self.write_message(&mut writer)?;
        Ok(Bytes::from(buf))
    }
}