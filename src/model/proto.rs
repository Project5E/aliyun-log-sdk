use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub struct Pair<'a> {
    pub(crate) key: Cow<'a, str>,
    pub(crate) value: Cow<'a, str>,
}

pub type Content<'a> = Pair<'a>;
pub type LogTag<'a> = Pair<'a>;

#[derive(Debug, PartialEq, Clone)]
pub struct Log<'a> {
    pub(crate) time: u32,
    pub(crate) contents: Vec<Content<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LogGroup<'a> {
    pub(crate) logs: Vec<Log<'a>>,
    // reserved: Option<Cow<'a, str>>,
    pub(crate) topic: Option<Cow<'a, str>>,
    pub(crate) source: Option<Cow<'a, str>>,
    pub(crate) log_tags: Vec<LogTag<'a>>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct LogGroupList<'a> {
    pub log_group_list: Vec<LogGroup<'a>>,
}