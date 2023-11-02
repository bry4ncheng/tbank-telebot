use anyhow::anyhow;
use axum::body::Bytes;
use futures_util::task::waker;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::Value;
use tracing::{warn, info};
use crate::models::customer::{AccountData, GetCustomerAccounts, GetCustomerDetails, HistoricalMonthlyBalanceBody, OnBoardCustomerData};
use crate::models::{TBankResponse, Error, ServiceResponseHeader, CustomerRequest};
use urlencoding::encode;
use crate::enums::beneficiary::BeneficiaryEnum;
use crate::models;
use crate::models::authentication::{RequestOTP, ServiceLoginOtpResponse};
use crate::models::chart::ChartBody;
use crate::models::transaction::{AddBeneficiaryBody, Beneficiaries, TransferBody, Beneficiary};


#[allow(dead_code)]
#[derive(Clone)]
pub struct TBankRepository {
    client: reqwest::Client,
    tbank_url: String,
    chart_url: String
}

impl TBankRepository {
    pub fn new(tbank_url: String, chart_url: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            tbank_url,
            chart_url
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
        let url = format!("{}?Header={}&Content={}&ConsumerID={}", self.tbank_url, encoded_header, encoded_content, "Teller");
        let req = self.client
            .post(url)
            .headers(headers)
            .send()
            .await;
        let res = match req {
            Ok(res) => {
                let temp = res.json::<Value>().await.unwrap();
                info!("{:?}", temp);
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
    pub async fn get_beneficiaries(self, body: CustomerRequest, _beneficiary_type: BeneficiaryEnum) -> anyhow::Result<Vec<models::transaction::Beneficiary>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
        let serde_body = serde_json::to_string(&body).unwrap();
        let consumer_id = encode("RIB").to_string();
        let encoded_header: String = encode(&serde_body).to_string();
        //let beneficiary_type = beneficiary_type.to_string();
        let content: String = r#"{"Content":{"accountGroup":"OTHER"}}"#.to_string();
        let encoded_content = encode(&content).to_string();
        let url = format!("{}?Header={}&Content={}&ConsumerID={}", self.tbank_url, encoded_header, encoded_content, consumer_id);
        let req = self.client
            .post(url)
            .headers(headers)
            .send()
            .await;
        let res = match req {
            Ok(res) => {
                let temp = res.json::<Value>().await.unwrap();
                let val = temp["Content"]["ServiceResponse"]["BeneficiaryList"]["Beneficiary"].clone();
                if val.is_array(){
                    let beneficiary_vec = match serde_json::from_value::<Beneficiaries>(temp["Content"]["ServiceResponse"]["BeneficiaryList"].clone()) {
                        Ok(res) => {
                            Ok(res.beneficiary)
                        }
                        Err(e) => {
                            warn!("{}", e);
                            Ok(vec![])
                        }
                    };
                    return beneficiary_vec
                }else{
                    let beneficiary_vec = match serde_json::from_value::<Beneficiary>(val) {
                        Ok(res) => {
                            Ok(vec![res])
                        }
                        Err(e) => {
                            warn!("{}", e);
                            Ok(vec![])
                        }
                    };
                    return beneficiary_vec
                }
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the RequestOTP API."))
            }
        };
        res
    }

    pub async fn add_beneficiary(self, body: CustomerRequest, content: AddBeneficiaryBody) -> anyhow::Result<String> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
        let serde_body = serde_json::to_string(&body).unwrap();
        let serde_content = serde_json::to_string(&content).unwrap();
        let encoded_header: String = encode(&serde_body).to_string();
        //let beneficiary_type = beneficiary_type.to_string();
        let encoded_content = encode(&serde_content).to_string();
        let url = format!("{}?Header={}&Content={}", self.tbank_url, encoded_header, encoded_content);
        info!("{}", url);
        let req = self.client
            .post(url)
            .headers(headers)
            .send()
            .await;
        let res = match req {
            Ok(res) => {
                let temp = res.json::<Value>().await.unwrap();
                let msg = temp["Content"]["ServiceResponse"]["ServiceRespHeader"]["ErrorText"].clone().to_string();
                Ok(msg)
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the RequestOTP API."))
            }
        };
        res
    }

    pub async fn transfer(self, body: CustomerRequest, content: TransferBody) -> anyhow::Result<String> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
        let serde_body = serde_json::to_string(&body).unwrap();
        let serde_content = serde_json::to_string(&content).unwrap();
        let consumer_id = encode("RIB").to_string();
        let encoded_header: String = encode(&serde_body).to_string();
        //let beneficiary_type = beneficiary_type.to_string();
        let encoded_content = encode(&serde_content).to_string();
        let url = format!("{}?Header={}&Content={}&ConsumerID={}", self.tbank_url, encoded_header, encoded_content, consumer_id);
        let req = self.client
            .post(url)
            .headers(headers)
            .send()
            .await;
        let res = match req {
            Ok(res) => {
                let temp = res.json::<Value>().await.unwrap();
                let status = temp["Content"]["ServiceResponse"]["ServiceRespHeader"]["ErrorText"].clone().to_string();
                Ok(status)
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the RequestOTP API."))
            }
        };
        res
    }

    pub async fn get_balance_chart(self, body: ChartBody) -> anyhow::Result<Bytes> {
        let serde_body = serde_json::to_string(&body).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
        let req = self.client
            .post(self.chart_url.clone())
            .headers(headers)
            .body(serde_body)
            .send()
            .await;
        let res = match req {
            Ok(res) => {
                let temp = res.bytes().await.unwrap();
                Ok(temp)
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the Chart API."))
            }
        };
        res
    }

    pub async fn get_monthly_balance_trend(self, body: CustomerRequest, content: HistoricalMonthlyBalanceBody) -> anyhow::Result<ChartBody> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
        let serde_body = serde_json::to_string(&body).unwrap();
        let serde_content = serde_json::to_string(&content).unwrap();
        println!("{:?}", serde_content);
        let consumer_id = encode("RIB").to_string();
        let encoded_header: String = encode(&serde_body).to_string();
        let encoded_content = encode(&serde_content).to_string();
        let url = format!("{}?Header={}&Content={}&ConsumerID={}", self.tbank_url, encoded_header, encoded_content, consumer_id);
        println!("{:?}", url);
        let req = self.client
            .post(url)
            .headers(headers)
            .send()
            .await;
        let res = match req {
            Ok(res) => {
                let temp = res.json::<Value>().await.unwrap();
                let chart = serde_json::from_value::<ChartBody>(temp["Content"]["ServiceResponse"]["TrendData"].clone());
                match chart {
                    Ok(res) => Ok(res),
                    Err(e) => {
                        warn!("{}", e);
                        Err(anyhow!("Something went wrong with the getMonthlyBalanceTrend API."))
                    }
                }
            }
            Err(e) => {
                warn!("{}", e);
                Err(anyhow!("Something went wrong with the getMonthlyBalanceTrend API."))
            }
        };
        res
    }
}
