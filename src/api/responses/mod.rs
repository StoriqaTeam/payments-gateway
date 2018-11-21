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
    pub erc20_approved: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl AccountsResponse {
    pub fn new(account: Account, balance: Amount, erc20_approved: bool) -> Self {
        Self {
            id: account.id,
            user_id: account.user_id,
            currency: account.currency,
            account_address: account.account_address.to_formatted(account.currency),
            name: account.name,
            balance: balance.to_string(),
            erc20_approved,
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
    pub from_value: String,
    pub from_currency: Currency,
    pub to_value: String,
    pub to_currency: Currency,
    pub fee: String,
    pub status: TransactionStatus,
    pub blockchain_tx_ids: Vec<BlockchainTransactionId>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Transaction> for TransactionsResponse {
    fn from(mut transaction: Transaction) -> Self {
        let from_currency = transaction.from_currency;
        transaction
            .from
            .iter_mut()
            .for_each(|info| info.blockchain_address = info.blockchain_address.to_formatted(from_currency));
        let to_currency = transaction.to_currency;
        transaction.to.blockchain_address = transaction.to.blockchain_address.to_formatted(to_currency);
        Self {
            id: transaction.id,
            from: transaction.from,
            to: transaction.to,
            from_currency: transaction.from_currency,
            from_value: transaction.from_value.to_string(),
            to_currency: transaction.to_currency,
            to_value: transaction.to_value.to_string(),
            fee: transaction.fee.to_string(),
            status: transaction.status,
            blockchain_tx_ids: transaction.blockchain_tx_ids,
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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeesResponse {
    pub currency: Currency,
    pub fees: Vec<FeeResponse>,
}

impl From<Fees> for FeesResponse {
    fn from(rate: Fees) -> Self {
        let fees = rate.fees.into_iter().map(From::from).collect();
        Self {
            currency: rate.currency,
            fees,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeResponse {
    pub value: String,
    pub estimated_time: u64,
}

impl From<Fee> for FeeResponse {
    fn from(fee: Fee) -> Self {
        Self {
            value: fee.value.to_string(),
            estimated_time: fee.estimated_time,
        }
    }
}
