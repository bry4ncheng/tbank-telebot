use anyhow::anyhow;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use tracing::warn;
use crate::models::customer::{ReplyOnboardCustomer, RequestOnboardCustomer};
use crate::models::TBankResponse;
use urlencoding::encode;
use crate::models::authentication::{ReplyOTP, RequestOTP};

#[allow(dead_code)]
#[derive(Clone)]
pub struct TBankRepository {
    client: reqwest::Client,
    tbank_url: String,
}

impl TBankRepository {
    pub fn new(tbank_url: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            tbank_url
        }
    }

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
