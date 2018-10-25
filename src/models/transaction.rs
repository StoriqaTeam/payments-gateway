use std::time::SystemTime;

use validator::Validate;

use models::*;

#[derive(Debug, Clone, Validate)]
pub struct CreateTransaction {
    pub from: AccountId,
    pub to: Receipt,
    pub to_type: ReceiptType,
    pub to_currency: Currency,
    pub value: Amount,
    pub fee: Amount,
}

impl Default for CreateTransaction {
    fn default() -> Self {
        Self {
            from: AccountId::generate(),
            to: Receipt::default(),
            to_type: ReceiptType::Account,
            to_currency: Currency::Eth,
            value: Amount::default(),
            fee: Amount::default(),
        }
    }
}

#[derive(Debug, Clone, Validate)]
pub struct Transaction {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionAddressInfo {
    pub account_id: Option<AccountId>,
    pub owner_name: Option<String>,
    pub blockchain_address: AccountAddress,
}
