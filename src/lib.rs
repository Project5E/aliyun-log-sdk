#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use chrono::Utc;
use digest::Digest;
use hmac::{Hmac, Mac, NewMac};
use md5::Md5;
use reqwest::header::{
    HeaderName, HeaderValue, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE,
    DATE, HOST, USER_AGENT,
};
use reqwest::{Client, Method, Url};
use sha1::Sha1;

mod error;
mod proto;

type HmacSha1 = Hmac<Sha1>;
pub use crate::error::{Error, Result};
use itertools::Itertools;
use std::str::FromStr;

pub const API_VERSION: &str = "0.6.0";
pub const SIGNATURE_METHOD: &str = "hmac-sha1";
pub const DEFAULT_CONTENT_TYPE: &str = "application/x-protobuf";
pub const USER_AGENT_VALUE: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
lazy_static! {
    static ref CONTENT_MD5: HeaderName = HeaderName::from_static("Content-MD5");
}

pub struct LogProducer {
    access_key: Box<str>,
    access_secret: Box<str>,
    endpoint: Box<str>,
    project: Box<str>,
    host: Box<str>,
    logstore: Box<str>,
    client: Client,
}

impl LogProducer {
    pub fn new<K, S, E, P, L>(
        access_key: K,
        access_secret: S,
        endpoint: E,
        project: P,
        logstore: L,
    ) -> Result<Self>
    where
        K: AsRef<str>,
        S: AsRef<str>,
        E: AsRef<str>,
        P: AsRef<str>,
        L: AsRef<str>,
    {
        Ok(Self {
            access_key: access_key.as_ref().into(),
            access_secret: access_secret.as_ref().into(),
            endpoint: endpoint.as_ref().into(),
            project: project.as_ref().into(),
            host: format!("{}.{}", project.as_ref(), endpoint.as_ref()).into_boxed_str(),
            logstore: logstore.as_ref().into(),
            client: reqwest::ClientBuilder::new().build()?,
        })
    }
    pub fn new_request<P>(&self, method: Method, path: P) -> Result<Request>
    where
        P: AsRef<str>,
    {
        let url = Url::from_str(&*format!("https://{}{}", self.endpoint, path.as_ref()))?;
        let mut request = reqwest::Request::new(method, url);
        let date = Utc::now().format("%a,%d%b%Y %H:%M:%S GMT").to_string();
        debug!("created request on {}", date);
        let headers = request.headers_mut();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
        headers.insert(DATE, HeaderValue::from_str(&*date).unwrap());
        headers.insert(HOST, HeaderValue::from_str(&*self.host).unwrap());
        headers.insert(
            HeaderName::from_static("x-log-apiversion"),
            HeaderValue::from_static(API_VERSION),
        );
        headers.insert(
            HeaderName::from_static("x-log-signaturemethod"),
            HeaderValue::from_static(SIGNATURE_METHOD),
        );

        Ok(Request {
            access_key: &*self.access_key,
            access_secret: &*self.access_secret,
            request,
            client: &self.client,
        })
    }
}

pub struct Request<'a> {
    access_key: &'a str,
    access_secret: &'a str,
    request: reqwest::Request,
    client: &'a Client,
}

impl std::ops::Deref for Request<'_> {
    type Target = reqwest::Request;

    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

impl std::ops::DerefMut for Request<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.request
    }
}

impl Request<'_> {
    pub async fn exec(mut self) -> Result<reqwest::Response> {
        let mut mac = HmacSha1::new_varkey(self.access_secret.as_bytes()).unwrap();
        // SignString = VERB + "\n"
        //              + CONTENT-MD5 + "\n"
        //              + CONTENT-TYPE + "\n"
        //              + DATE + "\n"
        //              + CanonicalizedLOGHeaders + "\n"
        //              + CanonicalizedResource
        let verb = self.method().as_str();
        mac.update(verb.as_bytes());
        mac.update(b"\n");

        if self.body().is_some() {
            let body = self.body().unwrap().as_bytes().unwrap();
            let length = body.len();
            let digest = Md5::digest(body);
            let md5 = self
                .headers_mut()
                .entry(CONTENT_MD5.clone())
                .or_insert_with(|| base64::encode(digest).parse().unwrap());
            mac.update(md5.as_ref());
            mac.update(b"\n");

            // Add CONTENT_LENGTH header
            self.headers_mut().insert(CONTENT_LENGTH, length.into());

            let content_type = self
                .headers_mut()
                .entry(CONTENT_TYPE)
                .or_insert_with(|| HeaderValue::from_static(DEFAULT_CONTENT_TYPE));
            mac.update(content_type.as_ref());
            mac.update(b"\n");
        } else {
            mac.update(b"\n\n");
        }

        let date = self.headers_mut().entry(DATE).or_insert_with(|| {
            let date = Utc::now().format("%a,%d%b%Y %H:%M:%S GMT").to_string();
            date.parse().unwrap()
        });

        mac.update(date.as_ref());
        mac.update(b"\n");
        // CanonicalizedLOGHeaders的构造方式如下：
        //     将所有以x-log和x-acs为前缀的HTTP请求头的名字转换成小写字母。
        //     将上一步得到的所有LOG自定义请求头按照字典顺序进行升序排序。
        //     删除请求头和内容之间分隔符两端出现的任何空格。
        //     将所有的头和内容用\n分隔符组合成最后的CanonicalizedLOGHeader。
        self.headers()
            .iter()
            .filter(|(key, _)| {
                key.as_str().starts_with("x-log") || key.as_str().starts_with("x-acs")
            })
            .sorted_by_key(|(key, _)| key.as_str())
            .map(|(key, value)| {
                format!(
                    "{}:{}",
                    key.as_str().to_ascii_lowercase(),
                    value.to_str().unwrap()
                )
            })
            .for_each(|next| {
                mac.update(next.as_bytes());
                mac.update(b"\n");
            });

        // CanonicalizedResource的构造方式如下：
        // a. 将CanonicalizedResource设置为空字符串" "。
        // b. 放入要访问的LOG资源，如/logstores/logstorename（如果没有logstorename则可不填写）。
        // c. 如果请求包含查询字符串QUERY_STRING，则在CanonicalizedResource字符串尾部添加?和查询字符串。
        //
        // QUERY_STRING是URL中请求参数按字典顺序排序后的字符串，其中参数名和值之间用=相隔组成字符串，并对参数名-值对按照字典顺序升序排序，然后以&符号连接构成字符串。其公式化描述如下：
        // QUERY_STRING = "KEY1=VALUE1" + "&" + "KEY2=VALUE2"

        let url = self.request.url();
        let path = url.path();
        debug!("-- path: {}", path);
        debug!("-- query: {:?}", url.query());
        mac.update(path.as_bytes());
        if url.query().is_some() {
            mac.update(b"?");
            let query_string = url
                .query_pairs()
                .map(|(key, value)| format!("{}={}", key, value))
                .sorted()
                .join("&");
            mac.update(query_string.as_bytes());
        }

        let authorization = base64::encode(mac.finalize().into_bytes());
        let authorization = format!("LOG {}:{}", self.access_key, authorization);
        self.headers_mut()
            .insert(AUTHORIZATION, authorization.parse().unwrap());

        Ok(self.client.execute(self.request).await?)
    }
}

// pub struct BatchLog {
//     inner: LogGroup,
// }
//
// impl BatchLog {
//     pub fn add_log(&mut self, log: Log) {}
// }

#[cfg(test)]
mod test {
    use crate::proto::{Content, Log, LogGroup};
    use crate::LogProducer;
    use bytes::BytesMut;
    use prost::Message;
    use reqwest::Method;
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
            reserved: None,
            topic: None,
            source: None,
            log_tags: vec![],
        };
        let mut buf = BytesMut::new();
        log_group.encode(&mut buf).unwrap();
        println!("{:?}", buf);
    }

    fn test_client() {
        //
    }

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
        let req = producer.new_request(Method::GET, "/logstores").unwrap();

        let result = rt.block_on(req.exec()).unwrap();
        let text = rt.block_on(result.text()).unwrap();
        debug!("{}", text)
    }
}
