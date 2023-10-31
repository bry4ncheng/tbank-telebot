pub enum BeneficiaryEnum {
    OWN,
    OTHER
}

impl BeneficiaryEnum {
    pub fn to_string(&self) -> String {
        match &self {
            BeneficiaryEnum::OWN => "OWN".to_string(),
            BeneficiaryEnum::OTHER => "OTHER".to_string()
        }
    }
}
