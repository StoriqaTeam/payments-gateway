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

#[derive(Debug, Clone, Validate)]
pub struct Transaction {
    pub from: AccountId,
    pub to: AccountId,
    pub to_currency: Currency,
    pub value: Amount,
    pub fee: Amount,
    pub blockchain_tx_id: Option<BlockchainTransactionId>,
}
