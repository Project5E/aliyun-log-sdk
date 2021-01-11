#[macro_use]
extern crate log;

use std::str::FromStr;

use chrono::Utc;
use digest::Digest;
use hmac::{Hmac, Mac, NewMac};
use itertools::Itertools;
use md5::Md5;
use reqwest::header::{
    HeaderValue, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, DATE, HOST, USER_AGENT,
};
use reqwest::{Client, Method, Url, Response, RequestBuilder};
use prost::Message;
use sha1::Sha1;

pub use crate::error::{Error, Result};
use crate::model::LogGroup;

mod error;
mod model;
mod headers {
    pub const LOG_API_VERSION: &str = "x-log-apiversion";
    pub const LOG_SIGNATURE_METHOD: &str = "x-log-signaturemethod";
    pub const LOG_BODY_RAW_SIZE: &str = "x-log-bodyrawsize";
    pub const CONTENT_MD5: &str = "content-md5";
}
use headers::*;
#[cfg(test)]
mod test;

type HmacSha1 = Hmac<Sha1>;

pub const API_VERSION: &str = "0.6.0";
pub const SIGNATURE_METHOD: &str = "hmac-sha1";
pub const DEFAULT_CONTENT_TYPE: &str = "application/x-protobuf";
pub const USER_AGENT_VALUE: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));


pub struct LogProducer {
    access_key: Box<str>,
    access_secret: Box<str>,
    endpoint: Box<str>,
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
            host: format!("{}.{}", project.as_ref(), endpoint.as_ref()).into_boxed_str(),
            logstore: logstore.as_ref().into(),
            client: reqwest::ClientBuilder::new().build()?,
        })
    }

    /// POST /logstores/logstoreName/shards/lb
    pub async fn put_logs_lb(&self, log_group: &LogGroup) -> Result<Response> {
        let mut buf = bytes::BytesMut::with_capacity(log_group.encoded_len());
        log_group.encode(&mut buf).unwrap(); // should not panic here
        let buf = buf.freeze();
        let request = self
            .new_request(Method::POST, format!("/logstores/{}/shards/lb", self.logstore))?
            .header(LOG_BODY_RAW_SIZE, buf.len())
            .body(buf);

        Ok(self.exec(request).await?)
    }

    fn new_request<P>(&self, method: Method, path: P) -> Result<RequestBuilder>
    where
        P: AsRef<str>,
    {
        let url = Url::from_str(&*format!("https://{}{}", self.endpoint, path.as_ref()))?;
        let date = Utc::now().format("%a,%d%b%Y %H:%M:%S GMT").to_string();
        debug!("created request on {}", date);
        let request = self.client.request(method, url)
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(DATE, date)
            .header(HOST, &*self.host)
            .header(LOG_API_VERSION, API_VERSION)
            .header(LOG_SIGNATURE_METHOD, SIGNATURE_METHOD);

        Ok(request)
    }

    async fn exec(&self, request: RequestBuilder) -> Result<reqwest::Response> {
        let mut request = request.build()?;

        let mut mac = HmacSha1::new_varkey(self.access_secret.as_bytes()).unwrap();
        // SignString = VERB + "\n"
        //              + CONTENT-MD5 + "\n"
        //              + CONTENT-TYPE + "\n"
        //              + DATE + "\n"
        //              + CanonicalizedLOGHeaders + "\n"
        //              + CanonicalizedResource
        let verb = request.method().as_str();
        debug!("-- method: {}", verb);
        mac.update(verb.as_bytes());
        mac.update(b"\n");


        if request.body().is_some() {
            debug!("-- body found");
            let body = request.body().unwrap().as_bytes().unwrap();
            let length = body.len();
            let digest = Md5::digest(body);
            let digest = hex::encode(digest).to_ascii_uppercase();
            debug!("-- content-md5: {}", digest);
            let md5 = request
                .headers_mut()
                .entry(CONTENT_MD5)
                .or_insert_with(|| digest.parse().unwrap());
            mac.update(md5.as_ref());
            mac.update(b"\n");

            // Add CONTENT_LENGTH header
            request.headers_mut().insert(CONTENT_LENGTH, length.into());

            let content_type = request
                .headers_mut()
                .entry(CONTENT_TYPE)
                .or_insert_with(|| HeaderValue::from_static(DEFAULT_CONTENT_TYPE));
            mac.update(content_type.as_ref());
            mac.update(b"\n");
        } else {
            mac.update(b"\n\n");
        }

        let date = request.headers_mut().entry(DATE).or_insert_with(|| {
            let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
            date.parse().unwrap()
        });

        mac.update(date.as_ref());
        mac.update(b"\n");
        // CanonicalizedLOGHeaders的构造方式如下：
        //     将所有以x-log和x-acs为前缀的HTTP请求头的名字转换成小写字母。
        //     将上一步得到的所有LOG自定义请求头按照字典顺序进行升序排序。
        //     删除请求头和内容之间分隔符两端出现的任何空格。
        //     将所有的头和内容用\n分隔符组合成最后的CanonicalizedLOGHeader。
        request.headers()
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
                debug!("-- header: {}", next);
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

        let url = request.url();
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
        request.headers_mut()
            .insert(AUTHORIZATION, authorization.parse().unwrap());

        Ok(self.client.execute(request).await?)
    }
}