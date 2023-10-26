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
#[serde(rename_all = "camelCase")]
pub struct GetCustomerAccounts<T> {
    #[serde(rename = "ServiceRespHeader")]
    pub service_response_header: Error,

    #[serde(rename = "AccountList")]
    pub account_list: Account<T>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Account<T> {
    pub account: T
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub interest_rate: String,
    #[serde(rename = "accountID")]
    pub account_id: String,
    pub parent_account_flag: String,
    pub balance: String,
    #[serde(rename = "productID")]
    pub product_id: String,
    pub current_status: String,
    pub currency: String,
    pub home_branch: String,
    pub account_open_date: String,
    pub maintenancehistory: Maintenancehistory,
    #[serde(rename = "officerID")]
    pub officer_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Maintenancehistory {
    pub last_transaction_branch: String,
    pub last_maintenance_officer: String,
}
