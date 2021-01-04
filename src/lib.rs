#[macro_use] extern crate log;

use crate::proto::{Log, LogGroup};

mod proto;

pub struct LogProducer {
    access_key: String,
    access_secret: String,
    region: String,
    project: String,
    logstore: String,
}

pub struct BatchLog {
    inner: LogGroup
}

impl BatchLog {
    fn add_log(&mut self, log: Log) {

    }
}

impl LogProducer {
    fn new() {
        let client = reqwest::ClientBuilder::new()

            .build();
    }
}

#[cfg(test)]
mod test {
    use std::time::SystemTime;
    use bytes::BytesMut;
    use prost::Message;
    use crate::proto::{Log, Content, LogGroup};

    #[test]
    fn test_proto() {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let contents = vec![Content {
            key: "test".to_string(),
            value: "ok".to_string()
        }];
        let log = Log {
            time: timestamp as u32,
            contents
        };
        let log_group = LogGroup {
            logs: vec![log],
            reserved: None,
            topic: None,
            source: None,
            log_tags: vec![]
        };
        let mut buf = BytesMut::new();
        log_group.encode(&mut buf).unwrap();
        println!("{:?}", buf);
    }
}
