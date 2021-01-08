/// Protobuf definition derive from https://help.aliyun.com/document_detail/29055.html
/// ```protobuf
/// message Log
/// {
///     required uint32 Time = 1;// UNIX Time Format
///     message Content
///     {
///         required string Key = 1;
///         required string Value = 2;
///     }
///     repeated Content Contents = 2;
/// }
///
/// message LogTag
/// {
///     required string Key = 1;
///     required string Value = 2;
/// }
///
/// message LogGroup
/// {
///     repeated Log Logs= 1;
///     optional string Reserved = 2; // reserved fields
///     optional string Topic = 3;
///     optional string Source = 4;
///     repeated LogTag LogTags = 6;
/// }
///
/// message LogGroupList
/// {
///     repeated LogGroup logGroupList = 1;
/// }
/// ```
use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub(crate) struct Log {
    /// UNIX Time Stamp
    #[prost(uint32, required, tag = "1")]
    time: u32,
    #[prost(message, repeated, tag = "2")]
    contents: Vec<Content>,
}
#[derive(Clone, PartialEq, Message)]
pub(crate) struct Pair {
    #[prost(string, required, tag = "1")]
    key: String,
    #[prost(string, required, tag = "2")]
    value: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct LogGroup {
    #[prost(message, repeated, tag = "1")]
    logs: Vec<Log>,
    /// reserved fields
    #[prost(string, optional, tag = "2")]
    _reserved: Option<String>,
    #[prost(string, optional, tag = "3")]
    topic: Option<String>,
    #[prost(string, optional, tag = "4")]
    source: Option<String>,
    #[prost(message, repeated, tag = "6")]
    log_tags: Vec<LogTag>,
}

#[derive(Clone, PartialEq, Message)]
pub struct LogGroupList {
    #[prost(message, repeated, tag = "1")]
    log_group_list: Vec<LogGroup>,
}

pub(crate) type Content = Pair;
pub(crate) type LogTag = Pair;

impl Pair {
    pub(crate) fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

impl Log {
    pub(crate) fn new(time: u32, contents: Vec<Content>) -> Self {
        Self { time, contents }
    }
}

#[cfg(test)]
mod test {
    use crate::model::proto::{Content, Log, LogGroup};
    use bytes::BytesMut;
    use prost::Message;
    use std::time::SystemTime;

    #[test]
    fn test_proto() {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let contents = vec![Content {
            key: "test".to_string(),
            value: "ok".to_string(),
        }];
        let log = Log {
            time: timestamp as u32,
            contents,
        };
        let log_group = LogGroup {
            logs: vec![log],
            _reserved: None,
            topic: None,
            source: None,
            log_tags: vec![],
        };
        let mut buf = BytesMut::new();
        log_group.encode(&mut buf).unwrap();
        println!("{:?}", buf);
    }
}
