use serde_json;
use validator::{ValidationError, ValidationErrors};

use models::*;

#[derive(Debug, Deserialize, Clone)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GraphQLError {
    pub message: String,
    pub path: Option<Vec<String>>,
    pub data: Option<GraphQLErrorData>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GraphQLErrorData {
    pub code: usize,
    pub details: GraphQLErrorDataDetailsEnum,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum GraphQLErrorDataDetailsEnum {
    Data(GraphQLErrorDataDetails),
    Str(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct GraphQLErrorDataDetails {
    status: String,
    code: Option<String>,
    description: Option<String>,
    message: Option<String>,
    payload: Option<String>,
}

pub fn get_error_payload(errors: Option<Vec<GraphQLError>>) -> Option<serde_json::Value> {
    errors.map(|errors| {
        let mut errors = errors.into_iter().fold(vec![], move |mut res, error| {
            if let Some(e) = error.data {
                if let GraphQLErrorDataDetailsEnum::Data(details) = e.details {
                    if let Some(payload) = details.payload {
                        let payload = serde_json::from_str(&payload).unwrap_or_default();
                        res.push(payload)
                    }
                }
                if e.code == 111 {
                    let mut errors = ValidationErrors::new();
                    let mut error = ValidationError::new("expired");
                    error.message = Some("JWT has expired.".into());
                    errors.add("token", error);
                    let payload = serde_json::to_value(&errors).unwrap_or_default();
                    res.push(payload)
                }
                if e.code == 112 {
                    let mut errors = ValidationErrors::new();
                    let mut error = ValidationError::new("revoked");
                    error.message = Some("JWT has been revoked.".into());
                    errors.add("token", error);
                    let payload = serde_json::to_value(&errors).unwrap_or_default();
                    res.push(payload)
                }
            }
            res
        });
        match errors.len() {
            0 => serde_json::Value::default(),
            1 => errors.pop().unwrap_or_default(),
            _ => serde_json::Value::Array(errors),
        }
    })
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateUser {
    pub create_user: User,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Me {
    pub me: Option<User>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserResponse {
    pub update_user: User,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetJWTByEmail {
    #[serde(rename = "getJWTByEmail")]
    pub get_jwt_by_email: Token,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetEmailVerify {
    pub verify_email: Token,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetJWTByProvider {
    #[serde(rename = "getJWTByProvider")]
    pub get_jwt_by_provider: Token,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetResetPassword {
    #[serde(rename = "requestPasswordReset")]
    pub request_password_reset: Reset,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetResendEmailVerify {
    #[serde(rename = "resendEmailVerificationLink")]
    pub resend_email_verify: Reset,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetRefreshJWT {
    #[serde(rename = "refreshJWT")]
    pub refresh_jwt: StoriqaJWT,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetRevokeJWT {
    #[serde(rename = "revokeJWT")]
    pub revoke_jwt: StoriqaJWT,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetResetPasswordApply {
    #[serde(rename = "applyPasswordReset")]
    pub apply_password_reset: ResetApply,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetChangePassword {
    #[serde(rename = "changePassword")]
    pub request_password_change: ResetApply,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Token {
    pub token: StoriqaJWT,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Reset {
    pub success: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResetApply {
    pub success: bool,
    pub token: StoriqaJWT,
}
