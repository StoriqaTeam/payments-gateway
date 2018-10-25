use std::time::SystemTime;

use models::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    pub id: AccountId,
    pub user_id: WorkspaceId,
    pub currency: Currency,
    pub address: AccountAddress,
    pub name: Option<String>,
    pub balance: Amount,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Default for AccountResponse {
    fn default() -> Self {
        Self {
            id: AccountId::generate(),
            user_id: WorkspaceId::generate(),
            currency: Currency::Eth,
            address: AccountAddress::default(),
            name: Some("new acc".to_string()),
            balance: Amount::default(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub id: TransactionId,
    pub from: Vec<TransactionAddressInfo>,
    pub to: Vec<TransactionAddressInfo>,
    pub currency: Currency,
    pub value: Amount,
    pub fee: Amount,
    pub status: TransactionStatus,
    pub blockchain_tx_id: Option<BlockchainTransactionId>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Default for TransactionResponse {
    fn default() -> Self {
        Self {
            id: TransactionId::generate(),
            from: vec![],
            to: vec![],
            currency: Currency::Eth,
            value: Amount::default(),
            fee: Amount::default(),
            status: TransactionStatus::Done,
            blockchain_tx_id: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}

impl From<TransactionResponse> for Transaction {
    fn from(transaction: TransactionResponse) -> Self {
        Self {
            id: transaction.id,
            from: transaction.from,
            to: transaction.to,
            currency: transaction.currency,
            value: transaction.value,
            fee: transaction.fee,
            status: transaction.status,
            blockchain_tx_id: transaction.blockchain_tx_id,
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
        }
    }
}
