# aliyun-log-sdk

> Rust version

**WARNING**: this implementation is experimental, api is **NOT** stable. Use it carefully.

example usage:
```rust
use aliyun_log_sdk::{Content, Log, LogProducer};

fn main() {
    // create a async runtime
    let rt = tokio::runtime::Runtime::new().unwrap();
    // create a new LogProducer instance
    let producer = LogProducer::new(
        env!("ACCESS_KEY"),
        env!("ACCESS_SECRET"),
        "cn-shenzhen.log.aliyuncs.com",
        "playground",
        "sdk-test",
    )
        .unwrap();

    // Create a vector of Log
    let mut records: Vec<Log> = Vec::new();
    // Create a new Log with default timestamp (now)
    let mut log: Log = Log::default();
    // Push K-V pairs to Log
    log.contents.push(Content::new("level", "INFO"));
    log.contents.push(Content::new("message", "startup"));
    // Add this log
    records.push(log);

    // Create LogGroup
    let log_group: LogGroup = LogGroup::new(records);
    // Send LogGroup to Aliyun
    let result = rt.block_on(producer.put_logs_lb(&log_group)).unwrap();
    let text = rt.block_on(result.text()).unwrap();
    println!("{}", text);
}
```