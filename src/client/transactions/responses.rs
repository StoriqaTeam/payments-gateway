use chrono::NaiveDateTime;

use models::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    pub id: AccountId,
    pub user_id: WorkspaceId,
    pub currency: Currency,
    pub address: AccountAddress,
    pub name: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for AccountResponse {
    fn default() -> Self {
        Self {
            id: AccountId::generate(),
            user_id: WorkspaceId::generate(),
            currency: Currency::Eth,
            address: AccountAddress::default(),
            name: Some("new acc".to_string()),
            created_at: ::chrono::Utc::now().naive_utc(),
            updated_at: ::chrono::Utc::now().naive_utc(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    pub account: AccountResponse,
    pub balance: Amount,
}

impl Default for BalanceResponse {
    fn default() -> Self {
        Self {
            account: AccountResponse::default(),
            balance: Amount::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub id: TransactionId,
    pub from: Vec<TransactionAddressInfo>,
    pub to: TransactionAddressInfo,
    pub from_value: Amount,
    pub from_currency: Currency,
    pub to_value: Amount,
    pub to_currency: Currency,
    pub fee: Amount,
    pub status: TransactionStatus,
    pub blockchain_tx_id: Option<BlockchainTransactionId>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for TransactionResponse {
    fn default() -> Self {
        Self {
            id: TransactionId::generate(),
            from: vec![],
            to: TransactionAddressInfo::default(),
            from_currency: Currency::Eth,
            from_value: Amount::default(),
            to_currency: Currency::Eth,
            to_value: Amount::default(),
            fee: Amount::default(),
            status: TransactionStatus::Done,
            blockchain_tx_id: None,
            created_at: ::chrono::Utc::now().naive_utc(),
            updated_at: ::chrono::Utc::now().naive_utc(),
        }
    }
}

impl From<TransactionResponse> for Transaction {
    fn from(transaction: TransactionResponse) -> Self {
        Self {
            id: transaction.id,
            from: transaction.from,
            to: transaction.to,
            from_currency: transaction.from_currency,
            from_value: transaction.from_value,
            to_currency: transaction.to_currency,
            to_value: transaction.to_value,
            fee: transaction.fee,
            status: transaction.status,
            blockchain_tx_id: transaction.blockchain_tx_id,
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
        }
    }
}
