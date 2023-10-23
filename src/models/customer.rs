use serde::{Deserialize, Serialize};
use crate::models::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestOnboardCustomer {
    #[serde(rename = "serviceName")]
    service_name: String,
    #[serde(rename = "IC_number")]
    ic_number: String,
    #[serde(rename = "familyName")]
    family_name: String,
    #[serde(rename = "givenName")]
    given_name: String,
    #[serde(rename = "dateOfBirth")]
    date_of_birth: String,
    gender: String,
    occupation: String,
    #[serde(rename = "streetAddress")]
    street_address: String,
    city: String,
    state: String,
    country: String,
    #[serde(rename = "postalCode")]
    postal_code: String,
    #[serde(rename = "emailAddress")]
    email_address: String,
    #[serde(rename = "countryCode")]
    country_code: String,
    #[serde(rename = "mobileNumber")]
    mobile_number: String,
    #[serde(rename = "preferredUserID")]
    preferred_user_id: String,
    currency: String,
    #[serde(rename = "bankID")]
    bank_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerData {
    #[serde(rename = "AccountID")]
    account_id: String,
    #[serde(rename = "branchID")]
    branch_id: Option<String>,
    #[serde(rename = "PIN")]
    pin: String,
    #[serde(rename = "CustomerID")]
    customer_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReplyOnboardCustomer {
    #[serde(rename = "ServiceRespHeader")]
    pub service_response_header: Error,
    #[serde(rename = "CustomerDetails")]
    pub customer_details: Option<CustomerData>
}