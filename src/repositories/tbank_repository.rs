
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

}
