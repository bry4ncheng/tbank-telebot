use anyhow::anyhow;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::Value;
use tracing::{warn};
use crate::models::customer::{AccountData, GetCustomerAccounts, GetCustomerDetails, OnBoardCustomerData};
use crate::models::{TBankResponse, Error, ServiceResponseHeader, CustomerRequest};
use urlencoding::encode;
use crate::models::authentication::{RequestOTP, ServiceLoginOtpResponse};


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

    //TBANK
    pub async fn onboard_customer(self, body: OnBoardCustomerData) -> anyhow::Result<serde_json::Value> {
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
                Ok(res.json::<serde_json::Value>().await.unwrap())
                // Ok(res.json::<TBankResponse<ReplyOnboardCustomer>>().await.unwrap())
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the OnboardCustomer API."))
            }
        };
        res
    }

    pub async fn request_otp(self, body: RequestOTP) -> anyhow::Result<TBankResponse<ServiceResponseHeader<Error>>> {
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
                Ok(res.json::<TBankResponse<ServiceResponseHeader<Error>>>().await.unwrap())
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the RequestOTP API."))
            }
        };
        res
    }

    pub async fn login_customer(self, body: CustomerRequest) -> anyhow::Result<TBankResponse<ServiceLoginOtpResponse>> {
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
                Ok(res.json::<TBankResponse<ServiceLoginOtpResponse>>().await.unwrap())
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the RequestOTP API."))
            }
        };
        res
    }

    pub async fn create_account(self, body: CustomerRequest) -> anyhow::Result<String> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
        let header = &serde_json::to_string(&body).unwrap();
        let encoded_header = encode(header);
        let encoded_content =
            r#"{"Content":{"productID":"101","openingBalance":"0","currency":"SGD","isRestricted":false,"isServiceChargeWaived":true,"isMinor":false,"makeDefaultAccount":false}}"#;
        let url = format!("{}?Header={}Content={}ConsumerID={}", self.tbank_url, encoded_header, encoded_content, "Teller");
        let req = self.client
            .post(url)
            .headers(headers)
            .send()
            .await;
        let res = match req {
            Ok(res) => {
                let temp = res.json::<Value>().await.unwrap();
                Ok(temp["Content"]["ServiceResponse"]["accountID"]["_content_"].to_string())
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the RequestOTP API."))
            }
        };
        res
    }

    pub async fn get_customer_accounts(self, body: CustomerRequest) -> anyhow::Result<Vec<AccountData>> {
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
                let mut vec_to_return = Vec::new();
                let json_val = res.json::<Value>().await.unwrap();
                let result_single = serde_json::from_value::<TBankResponse<GetCustomerAccounts<AccountData>>>(json_val.clone());
                if result_single.is_ok() {
                    vec_to_return.push(result_single.unwrap().content.service_response.account_list.account)
                }else{
                    let result_multiple = serde_json::from_value::<TBankResponse<GetCustomerAccounts<Vec<AccountData>>>>(json_val.clone());
                    if result_multiple.is_ok(){
                        vec_to_return = result_multiple.unwrap().content.service_response.account_list.account;
                    }else{
                        return Err(anyhow!("Something went wrong with the RequestOTP API."))
                    }
                }
                Ok(vec_to_return)
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the RequestOTP API."))
            }
        };
        res
    }

    pub async fn get_customer_details(self, body: CustomerRequest) -> anyhow::Result<TBankResponse<GetCustomerDetails>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
        let serde_body = serde_json::to_string(&body).unwrap();
        let consumer_id = encode("RIB").to_string();
        let encoded_header: String = encode(&serde_body).to_string();
        let url = format!("{}?Header={}ConsumerID={}", self.tbank_url, encoded_header, consumer_id);
        let req = self.client
            .post(url)
            .headers(headers)
            .send()
            .await;
        let res = match req {
            Ok(res) => {
                
                Ok(res.json::<TBankResponse<GetCustomerDetails>>().await.unwrap())
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the RequestOTP API."))
            }
        };
        res
    }
}
