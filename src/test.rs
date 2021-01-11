use crate::model::{LogGroup, Log};
use crate::LogProducer;
use reqwest::Method;

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
    let mut log_group = LogGroup::default();
    let record = Log::default();
    log_group.add_log(record);
    let result = rt.block_on(producer.put_logs_lb(&log_group)).unwrap();
    let text = rt.block_on(result.text()).unwrap();
    debug!("{}", text)
}
