use serde_json;

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
    pub details: GraphQLErrorDataDetails,
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
                if let Some(payload) = e.details.payload {
                    let payload = serde_json::from_str(&payload).unwrap_or_default();
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
    pub me: User,
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
pub struct GetResetPasswordApply {
    #[serde(rename = "applyPasswordReset")]
    pub apply_password_reset: ResetApply,
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
