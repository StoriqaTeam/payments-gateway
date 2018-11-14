use chrono::NaiveDateTime;

use super::{Error, ErrorContext, ErrorKind};
use models::*;
use prelude::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostSessionsRequest {
    pub email: String,
    pub password: Password,
    pub device_type: DeviceType,
    pub device_os: Option<String>,
    pub device_id: Option<DeviceId>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostUsersResetPasswordRequest {
    pub email: String,
    pub device_type: DeviceType,
}

impl From<PostUsersResetPasswordRequest> for ResetPassword {
    fn from(req: PostUsersResetPasswordRequest) -> Self {
        Self {
            email: req.email,
            device: req.device_type,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostUsersChangePasswordRequest {
    pub new_password: Password,
    pub old_password: Password,
}

impl From<PostUsersChangePasswordRequest> for ChangePassword {
    fn from(req: PostUsersChangePasswordRequest) -> Self {
        Self {
            new_password: req.new_password,
            old_password: req.old_password,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostUsersConfirmResetPasswordRequest {
    pub token: String,
    pub password: Password,
}

impl From<PostUsersConfirmResetPasswordRequest> for ResetPasswordConfirm {
    fn from(req: PostUsersConfirmResetPasswordRequest) -> Self {
        Self {
            token: req.token,
            password: req.password,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostSessionsOauthRequest {
    pub oauth_token: OauthToken,
    pub oauth_provider: Provider,
    pub device_type: DeviceType,
    pub device_os: Option<String>,
    pub device_id: Option<DeviceId>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostUsersRequest {
    pub email: String,
    pub password: Password,
    pub first_name: String,
    pub last_name: String,
    pub device_type: DeviceType,
    pub device_os: String,
    pub device_id: DeviceId,
    pub public_key: DevicePublicKey,
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
            device_os: req.device_os,
            device_id: req.device_id,
            public_key: req.public_key,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PutUsersRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}

impl From<PutUsersRequest> for UpdateUser {
    fn from(req: PutUsersRequest) -> Self {
        Self {
            first_name: req.first_name,
            last_name: req.last_name,
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
pub struct PostUsersConfirmAddDeviceRequest {
    pub token: DeviceConfirmToken,
    pub public_key: DevicePublicKey,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostUsersAddDeviceRequest {
    pub device_id: DeviceId,
    pub device_os: String,
    pub public_key: DevicePublicKey,
    pub email: String,
    pub password: Password,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostAccountsRequest {
    pub id: AccountId,
    pub currency: Currency,
    pub name: String,
    pub callback_url: Option<String>,
}

impl From<(PostAccountsRequest, UserId)> for CreateAccount {
    fn from(req: (PostAccountsRequest, UserId)) -> Self {
        Self {
            id: req.0.id,
            name: req.0.name,
            currency: req.0.currency,
            callback_url: req.0.callback_url,
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
    pub id: TransactionId,
    pub user_id: UserId,
    pub from: AccountId,
    pub to: Receipt,
    pub to_type: ReceiptType,
    pub to_currency: Currency,
    pub value: String,
    pub fee: String,
    pub value_currency: Currency,
    pub hold_until: Option<NaiveDateTime>,
    pub exchange_id: Option<ExchangeId>,
    pub exchange_rate: Option<f64>,
}

impl PostTransactionsRequest {
    pub fn to_create(self) -> Result<CreateTransaction, Error> {
        let req_ = self.clone();
        let value = u128::from_str_radix(&self.value, 10).map_err(ectx!(try ErrorContext::RequestJson, ErrorKind::BadRequest => req_))?;
        let value = Amount::new(value);
        let req_ = self.clone();
        let fee = u128::from_str_radix(&self.fee, 10).map_err(ectx!(try ErrorContext::RequestJson, ErrorKind::BadRequest => req_))?;
        let fee = Amount::new(fee);
        Ok(CreateTransaction {
            id: self.id,
            from: self.from,
            to: self.to,
            to_type: self.to_type,
            to_currency: self.to_currency,
            value_currency: self.value_currency,
            value,
            fee,
            exchange_id: self.exchange_id,
            exchange_rate: self.exchange_rate,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetUsersTransactionsParams {
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostRateRequest {
    pub id: ExchangeId,
    pub from: Currency,
    pub to: Currency,
    pub amount: Amount,
    pub amount_currency: Currency,
}

impl From<PostRateRequest> for GetRate {
    fn from(req: PostRateRequest) -> Self {
        Self {
            id: req.id,
            from: req.from,
            to: req.to,
            amount: req.amount,
            amount_currency: req.amount_currency,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostFeesRequest {
    pub from_currency: Currency,
    pub to_currency: Currency,
}

impl From<PostFeesRequest> for GetFees {
    fn from(req: PostFeesRequest) -> Self {
        Self {
            from_currency: req.from_currency,
            to_currency: req.to_currency,
        }
    }
}
