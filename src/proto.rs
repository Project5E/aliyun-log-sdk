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
pub struct Log {
    /// UNIX Time Stamp
    #[prost(uint32, required, tag="1")]
    pub time: u32,
    #[prost(message, repeated, tag="2")]
    pub contents: Vec<Pairs>,
}
#[derive(Clone, PartialEq, Message)]
pub struct Pairs {
    #[prost(string, required, tag="1")]
    pub key: String,
    #[prost(string, required, tag="2")]
    pub value: String,
}
pub type Content = Pairs;
pub type LogTag = Pairs;

#[derive(Clone, PartialEq, Message)]
pub struct LogGroup {
    #[prost(message, repeated, tag="1")]
    pub logs: Vec<Log>,
    /// reserved fields
    #[prost(string, optional, tag="2")]
    pub reserved: Option<String>,
    #[prost(string, optional, tag="3")]
    pub topic: Option<String>,
    #[prost(string, optional, tag="4")]
    pub source: Option<String>,
    #[prost(message, repeated, tag="6")]
    pub log_tags: Vec<LogTag>,
}
#[derive(Clone, PartialEq, Message)]
pub struct LogGroupList {
    #[prost(message, repeated, tag="1")]
    pub log_group_list: Vec<LogGroup>,
}
