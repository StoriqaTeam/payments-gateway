use super::storiqa_jwt::StoriqaJWT;
use models::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    pub token: StoriqaJWT,
    pub user_id: UserId,
}
