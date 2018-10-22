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
    pub from: AccountId,
    pub to: AccountId,
    pub to_currency: Currency,
    pub value: Amount,
    pub blockchain_tx_id: Option<BlockchainTransactionId>,
    pub fee: Amount,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}
