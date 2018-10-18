use models::*;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    pub id: AccountId,
    pub user_id: WorkspaceId,
    pub currency: Currency,
    pub name: String,
}

impl From<(CreateAccount, WorkspaceId)> for CreateAccountRequest {
    fn from(req: (CreateAccount, WorkspaceId)) -> Self {
        Self {
            id: req.0.id,
            name: req.0.name,
            currency: req.0.currency,
            user_id: req.1,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetUsersAccountsParams {
    pub limit: i64,
    pub offset: AccountId,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionRequest {
    pub user_id: WorkspaceId,
    pub from: AccountId,
    pub to: Receipt,
    pub to_type: ReceiptType,
    pub to_currency: Currency,
    pub value: Amount,
    pub fee: Amount,
}

impl From<(CreateTransaction, WorkspaceId)> for CreateTransactionRequest {
    fn from(req: (CreateTransaction, WorkspaceId)) -> Self {
        Self {
            user_id: req.1,
            from: req.0.from,
            to: req.0.to,
            to_type: req.0.to_type,
            to_currency: req.0.to_currency,
            value: req.0.value,
            fee: req.0.fee,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetUsersTransactionsParams {
    pub limit: i64,
    pub offset: TransactionId,
}
