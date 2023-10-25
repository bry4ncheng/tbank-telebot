use anyhow::anyhow;
use bb8_redis::{bb8, RedisConnectionManager};
use bb8_redis::redis::{AsyncCommands, RedisError};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::Value;
use tracing::warn;
use crate::models::customer::{ReplyOnboardCustomer, RequestOnboardCustomer};
use crate::models::TBankResponse;
use urlencoding::encode;
use crate::models::authentication::{ReplyOTP, RequestOTP};

const REDIS_PREFIX: &str = "usr";

#[allow(dead_code)]
#[derive(Clone)]
pub struct TBankRepository {
    client: reqwest::Client,
    redis_client: bb8::Pool<RedisConnectionManager>,
    tbank_url: String,
}

impl TBankRepository {
    pub async fn new(tbank_url: String, redis_url: String) -> Self {
        let client = reqwest::Client::new();
        let redis_manager = RedisConnectionManager::new(redis_url.clone()).unwrap();
        let redis_pool = bb8::Pool::builder().build(redis_manager).await.unwrap();
        Self {
            client,
            redis_client: redis_pool,
            tbank_url
        }
    }

    //REDIS
    pub async fn get_credentials_from_redis(self, key: &String) -> anyhow::Result<Value> {
        let mut redis_conn = self.redis_client.get().await.unwrap();
        let redis_key = format!("{}:{}", REDIS_PREFIX, key);
        let res: String = redis_conn.get(redis_key.clone()).await?;
        //Up to u if u need this in use case @zaki
        //delete entry after use
        //let _del_res = redis_conn.del(redis_key).await?;
        let data = match serde_json::from_str::<Value>(&res) {
            Ok(data) => Ok(data),
            Err(e) => {
                warn!("Something went wrong while deserializing: {:?}", e);
                Err(anyhow!("Something went wrong!"))
            }
        };
        data
    }

    pub async fn set_user_in_redis(self, key: &String, value: String) -> anyhow::Result<()> {
        let mut redis_conn = self.redis_client.get().await?;
        //1 day ttl
        let res : Result<(), RedisError> = redis_conn.set_ex(format!("{}:{}", REDIS_PREFIX, key.clone()), value, 86400).await;
        return match res {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Something went wrong populating redis: {:?}", e);
                Err(anyhow!("Something went wrong!"))
            }
        };
    }

    //TBANK
    pub async fn onboard_customer(self, body: RequestOnboardCustomer) -> anyhow::Result<TBankResponse<ReplyOnboardCustomer>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
        let serde_body = serde_json::to_string(&body).unwrap();
        let encoded_header = encode(r#"{"serviceName":"onboardCustomer","userID":"","PIN":"","OTP":""}"#).to_string();
        let encoded_content = encode(&serde_body).to_string();
        let url = format!("{}?Header={}Content={}", self.tbank_url, encoded_header, encoded_content);
        let req = self.client
            .post(url)
            .headers(headers)
            .send()
            .await;
        let res = match req {
            Ok(res) => {
                Ok(res.json::<TBankResponse<ReplyOnboardCustomer>>().await.unwrap())
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the OnboardCustomer API."))
            }
        };
        res
    }

    pub async fn request_otp(self, body: RequestOTP) -> anyhow::Result<TBankResponse<ReplyOTP>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
        let serde_body = serde_json::to_string(&body).unwrap();
        let consumer_id = encode("RIB").to_string();
        let encoded_header = encode(&serde_body).to_string();
        let url = format!("{}?Header={}ConsumerID={}", self.tbank_url, encoded_header, consumer_id);
        let req = self.client
            .post(url)
            .headers(headers)
            .send()
            .await;
        let res = match req {
            Ok(res) => {
                Ok(res.json::<TBankResponse<ReplyOTP>>().await.unwrap())
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the RequestOTP API."))
            }
        };
        res
    }
}
