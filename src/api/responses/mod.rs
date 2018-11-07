use chrono::NaiveDateTime;

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
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Account> for AccountsResponse {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            user_id: account.user_id,
            currency: account.currency,
            account_address: account.account_address.to_formatted(account.currency),
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
    pub id: TransactionId,
    pub from: Vec<TransactionAddressInfo>,
    pub to: TransactionAddressInfo,
    pub currency: Currency,
    pub value: String,
    pub fee: String,
    pub status: TransactionStatus,
    pub blockchain_tx_id: Option<BlockchainTransactionId>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Transaction> for TransactionsResponse {
    fn from(mut transaction: Transaction) -> Self {
        let currency = transaction.currency;
        transaction
            .from
            .iter_mut()
            .for_each(|info| info.blockchain_address = info.blockchain_address.to_formatted(currency));
        transaction.to.blockchain_address = transaction.to.blockchain_address.to_formatted(currency);
        Self {
            id: transaction.id,
            from: transaction.from,
            to: transaction.to,
            currency: transaction.currency,
            value: transaction.value.to_string(),
            fee: transaction.fee.to_string(),
            status: transaction.status,
            blockchain_tx_id: transaction.blockchain_tx_id,
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RateResponse {
    pub id: ExchangeId,
    pub from: Currency,
    pub to: Currency,
    pub amount: Amount,
    pub expiration: NaiveDateTime,
    pub rate: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Rate> for RateResponse {
    fn from(rate: Rate) -> Self {
        Self {
            id: rate.id,
            from: rate.from,
            to: rate.to,
            amount: rate.amount,
            expiration: rate.expiration,
            rate: rate.rate,
            created_at: rate.created_at,
            updated_at: rate.updated_at,
        }
    }
}
