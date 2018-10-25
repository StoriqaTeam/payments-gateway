use std::time::SystemTime;

use models::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostSessionsRequest {
    pub email: String,
    pub password: Password,
    pub device_type: DeviceType,
    pub device_os: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostSessionsOauthRequest {
    pub oauth_token: OauthToken,
    pub oauth_provider: Provider,
    pub device_type: DeviceType,
    pub device_os: Option<String>,
    pub device_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostUsersRequest {
    pub email: String,
    pub password: Password,
    pub first_name: String,
    pub last_name: String,
    pub device_type: DeviceType,
    pub device_os: Option<String>,
    pub device_id: Option<String>,
    pub phone: Option<String>,
}

impl From<PostUsersRequest> for NewUser {
    fn from(req: PostUsersRequest) -> Self {
        Self {
            email: req.email,
            password: req.password,
            first_name: req.first_name,
            last_name: req.last_name,
            device_type: req.device_type,
            phone: req.phone,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostUsersConfirmEmailRequest {
    pub email_confirm_token: EmailConfirmToken,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostAccountsRequest {
    pub id: AccountId,
    pub currency: Currency,
    pub name: String,
}

impl From<(PostAccountsRequest, UserId)> for CreateAccount {
    fn from(req: (PostAccountsRequest, UserId)) -> Self {
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
pub struct PutAccountsRequest {
    pub name: String,
}

impl From<PutAccountsRequest> for UpdateAccount {
    fn from(req: PutAccountsRequest) -> Self {
        Self { name: req.name }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetUsersAccountsParams {
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostTransactionsRequest {
    pub user_id: UserId,
    pub from: AccountId,
    pub to: Receipt,
    pub to_type: ReceiptType,
    pub to_currency: Currency,
    pub value: Amount,
    pub fee: Amount,
    pub hold_until: Option<SystemTime>,
}

impl From<PostTransactionsRequest> for CreateTransaction {
    fn from(req: PostTransactionsRequest) -> Self {
        Self {
            from: req.from,
            to: req.to,
            to_type: req.to_type,
            to_currency: req.to_currency,
            value: req.value,
            fee: req.fee,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetUsersTransactionsParams {
    pub limit: i64,
    pub offset: i64,
}
