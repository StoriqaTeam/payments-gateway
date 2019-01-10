use chrono::NaiveDateTime;

use validator::Validate;

use models::*;

#[derive(Debug, Clone, Validate)]
pub struct CreateTransaction {
    pub id: TransactionId,
    pub from: AccountId,
    pub to: Receipt,
    pub to_type: ReceiptType,
    pub to_currency: Currency,
    pub value: Amount,
    pub fee: Amount,
    pub value_currency: Currency,
    pub exchange_id: Option<ExchangeId>,
    pub exchange_rate: Option<f64>,
    pub fiat_value: Option<String>,
    pub fiat_currency: Option<String>,
}

impl Default for CreateTransaction {
    fn default() -> Self {
        Self {
            id: TransactionId::generate(),
            from: AccountId::generate(),
            to: Receipt::default(),
            to_type: ReceiptType::Account,
            to_currency: Currency::Eth,
            value_currency: Currency::Eth,
            value: Amount::default(),
            fee: Amount::default(),
            exchange_id: None,
            exchange_rate: None,
            fiat_value: None,
            fiat_currency: None,
        }
    }
}

#[derive(Debug, Clone, Validate, Deserialize)]
pub struct Transaction {
    pub id: TransactionId,
    pub from: Vec<TransactionAddressInfo>,
    pub to: TransactionAddressInfo,
    pub from_value: Amount,
    pub from_currency: Currency,
    pub to_value: Amount,
    pub to_currency: Currency,
    pub fee: Amount,
    pub status: TransactionStatus,
    pub blockchain_tx_ids: Vec<BlockchainTransactionId>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub fiat_value: Option<String>,
    pub fiat_currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TransactionAddressInfo {
    pub account_id: Option<AccountId>,
    pub owner_name: Option<String>,
    pub blockchain_address: AccountAddress,
}
