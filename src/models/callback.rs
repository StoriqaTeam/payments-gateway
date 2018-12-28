use models::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Callback {
    pub url: String,
    pub transaction_id: TransactionId,
    pub amount_captured: String,
    pub currency: Currency,
    pub address: AccountAddress,
    pub account_id: AccountId,
}

impl Default for Callback {
    fn default() -> Self {
        Self {
            url: String::default(),
            transaction_id: TransactionId::generate(),
            amount_captured: String::default(),
            currency: Currency::Eth,
            address: AccountAddress::default(),
            account_id: AccountId::generate(),
        }
    }
}

impl Callback {
    pub fn new(
        url: String,
        transaction_id: TransactionId,
        amount_captured: String,
        currency: Currency,
        address: AccountAddress,
        account_id: AccountId,
    ) -> Self {
        Self {
            url,
            transaction_id,
            amount_captured,
            currency,
            address,
            account_id,
        }
    }
}
