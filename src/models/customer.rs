
use serde::{Deserialize, Serialize};
use crate::models::Error;
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
#[serde(rename_all = "camelCase")]
pub struct GetCustomerDetails {
    #[serde(rename = "ServiceRespHeader")]
    pub service_response_header: Error,

    #[serde(rename = "CDMCustomer")]
    pub cdm_customer: CdmCustomer,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CdmCustomer {
    pub address: Address,
    pub phone: Phone,
    pub profile: Profile,
    pub family_name: String,
    pub given_name: String,
    pub certificate: Certificate,
    pub tax_identifier: String,
    pub cellphone: Cellphone,
    pub maintenacehistory: Maintenacehistory,
    pub date_of_birth: String,
    pub customer: Customer,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub country: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub street_address1: Option<String>,
    pub street_address2: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Phone {
    pub area_code: Option<String>,
    pub country_code: Option<String>,
    pub local_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub customer_type: Option<String>,
    #[serde(rename = "bankID")]
    pub bank_id: Option<String>,
    pub occupation: Option<String>,
    pub nationality: Option<String>,
    pub gender: Option<String>,
    pub is_billing_org: Option<String>,
    pub is_merchant: Option<String>,
    pub ethnic_group: Option<String>,
    pub fax: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    pub certificate_expiry_date: Option<String>,
    pub certificate_issuer: Option<String>,
    pub certificate_no: Option<String>,
    pub certificate_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cellphone {
    pub phone_number: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Maintenacehistory {
    pub registration_date: Option<String>,
    pub last_maintenance_teller_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
    #[serde(rename = "customerID")]
    pub customer_id: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Account<T> {
    pub account: T
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnBoardCustomerData {
    #[serde(rename = "serviceName")]
    pub service_name: String,
    #[serde(rename = "IC_number")]
    pub ic_number: String,
    #[serde(rename = "familyName")]
    pub family_name: String,
    #[serde(rename = "givenName")]
    pub given_name: String,
    #[serde(rename = "dateOfBirth")]
    pub date_of_birth: String,
    pub gender: String,
    pub occupation: String,
    #[serde(rename = "streetAddress")]
    pub street_address: String,
    pub city: String,
    pub state: String,
    pub country: String,
    #[serde(rename = "postalCode")]
    pub postal_code: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    #[serde(rename = "mobileNumber")]
    pub mobile_number: String,
    #[serde(rename = "preferredUserID")]
    pub preferred_user_id: String,
    pub currency: String,
    #[serde(rename = "bankID")]
    pub bank_id: String,

}

#[derive(Debug, Serialize, Deserialize)]
pub struct OnBoardCustomerError {
    #[serde(rename = "ErrorText")]
    pub error_text: Option<String>,
    #[serde(rename = "ErrorDetails")]
    pub error_details: Option<String>,
    #[serde(rename = "GlobalErrorID")]
    pub global_error_id: Option<String>,
    #[serde(rename = "AccountID")]
    pub account_id: Option<String>,
    #[serde(rename = "CustomerID")]
    pub customer_id: Option<String>,
    #[serde(rename = "branchID")]
    pub branch_id: Option<String>,
    #[serde(rename = "PIN")]
    pub pin: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Maintenancehistory {
    pub last_transaction_branch: String,
    pub last_maintenance_officer: String,
}
