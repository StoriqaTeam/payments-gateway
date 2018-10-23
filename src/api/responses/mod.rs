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
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl From<Account> for AccountsResponse {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            user_id: account.user_id,
            currency: account.currency,
            account_address: account.account_address,
            name: account.name,
            balance: account.balance.to_string(),
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionsResponse {
    pub from: AccountId,
    pub to: AccountId,
    pub to_currency: Currency,
    pub value: String,
    pub blockchain_tx_id: Option<BlockchainTransactionId>,
}

impl From<Transaction> for TransactionsResponse {
    fn from(transaction: Transaction) -> Self {
        Self {
            from: transaction.from,
            to: transaction.to,
            to_currency: transaction.to_currency,
            value: transaction.value.to_string(),
            blockchain_tx_id: transaction.blockchain_tx_id,
        }
    }
}
