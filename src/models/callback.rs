use models::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Callback {
    pub url: String,
    pub amount_captured: Amount,
    pub currency: Currency,
    pub address: AccountAddress,
}

impl Default for Callback {
    fn default() -> Self {
        Self {
            url: String::default(),
            amount_captured: Amount::default(),
            currency: Currency::Eth,
            address: AccountAddress::default(),
        }
    }
}
