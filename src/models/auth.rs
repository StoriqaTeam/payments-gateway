use super::storiqa_jwt::StoriqaJWT;
use super::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    token: StoriqaJWT,
    user: User,
}
