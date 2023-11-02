use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ChartBody {
    #[serde(rename = "MonthEndBalance")]
    MonthEndBalance: Vec<BalanceRecord>,
    #[serde(rename = "CurrentMonth")]
    CurrentMonth: BalanceRecord,
}

#[derive(Debug, Deserialize, Serialize)]
struct BalanceRecord {
    #[serde(rename = "Year_Month")]
    Year_Month: String,
    #[serde(rename = "Balance")]
    Balance: String,
}