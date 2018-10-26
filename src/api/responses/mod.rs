use std::time::SystemTime;

use models::*;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostSessionsResponse {
    pub token: StoriqaJWT,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountsResponse {
    pub id: AccountId,
    pub user_id: UserId,
    pub currency: Currency,
    pub account_address: AccountAddress,
    pub name: String,
    pub balance: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<Account> for AccountsResponse {
    fn from(account: Account) -> Self {
        let created_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let updated_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id: account.id,
            user_id: account.user_id,
            currency: account.currency,
            account_address: account.account_address,
            name: account.name,
            balance: account.balance.to_string(),
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionsResponse {
    pub id: TransactionId,
    pub from: Vec<TransactionAddressInfo>,
    pub to: TransactionAddressInfo,
    pub currency: Currency,
    pub value: String,
    pub fee: String,
    pub status: TransactionStatus,
    pub blockchain_tx_id: Option<BlockchainTransactionId>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl From<Transaction> for TransactionsResponse {
    fn from(transaction: Transaction) -> Self {
        let created_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let updated_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id: transaction.id,
            from: transaction.from,
            to: transaction.to,
            currency: transaction.currency,
            value: transaction.value.to_string(),
            fee: transaction.fee.to_string(),
            status: transaction.status,
            blockchain_tx_id: transaction.blockchain_tx_id,
            created_at,
            updated_at,
        }
    }
}
