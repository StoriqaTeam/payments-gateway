use models::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    pub currency: Currency,
    pub fees: Vec<Fee>,
}

impl Default for Fees {
    fn default() -> Self {
        Self {
            currency: Currency::Eth,
            fees: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fee {
    pub value: Amount,
    pub estimated_time: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFees {
    pub currency: Currency,
    pub account_address: AccountAddress,
}

impl Default for GetFees {
    fn default() -> Self {
        Self {
            currency: Currency::Btc,
            account_address: AccountAddress::default(),
        }
    }
}
