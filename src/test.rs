use crate::model::{Log, LogGroup};
use crate::LogProducer;

#[test]
fn test_new_request() {
    env_logger::init();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let producer = LogProducer::new(
        env!("ACCESS_KEY"),
        env!("ACCESS_SECRET"),
        "cn-shenzhen.log.aliyuncs.com",
        "playground",
        "sdk-test",
    )
    .unwrap();
    //let req = producer.new_request(Method::GET, "/logstores").unwrap();
    let mut records = Vec::new();
    records.push(Log::default());
    let log_group = LogGroup::new(records);
    let result = rt.block_on(producer.put_logs_lb(&log_group)).unwrap();
    let text = rt.block_on(result.text()).unwrap();
    debug!("{}", text)
}
