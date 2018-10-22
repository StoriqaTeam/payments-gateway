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
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    pub balance: Amount,
    pub currency: Currency,
}

impl Default for BalanceResponse {
    fn default() -> Self {
        Self {
            balance: Amount::default(),
            currency: Currency::Eth,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalancesResponse {
    #[serde(flatten)]
    pub data: Vec<BalanceResponse>,
}

impl Default for BalancesResponse {
    fn default() -> Self {
        Self {
            data: vec![BalanceResponse::default()],
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub id: TransactionId,
    pub user_id: UserId,
    pub dr_account_id: AccountId,
    pub cr_account_id: AccountId,
    pub currency: Currency,
    pub value: Amount,
    pub fee: Amount,
    pub status: TransactionStatus,
    pub blockchain_tx_id: Option<BlockchainTransactionId>,
    pub hold_until: Option<SystemTime>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Default for TransactionResponse {
    fn default() -> Self {
        Self {
            id: TransactionId::generate(),
            user_id: UserId::generate(),
            dr_account_id: AccountId::generate(),
            cr_account_id: AccountId::generate(),
            currency: Currency::Stq,
            value: Amount::default(),
            fee: Amount::default(),
            status: TransactionStatus::Pending,
            blockchain_tx_id: None,
            hold_until: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}

impl From<TransactionResponse> for Transaction {
    fn from(resp: TransactionResponse) -> Self {
        Self {
            from: resp.dr_account_id,
            to: resp.cr_account_id,
            to_currency: resp.currency,
            value: resp.value,
            blockchain_tx_id: resp.blockchain_tx_id,
            fee: resp.fee,
            created_at: resp.created_at,
            updated_at: resp.updated_at,
        }
    }
}
