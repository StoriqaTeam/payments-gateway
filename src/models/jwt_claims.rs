use models::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTClaims {
    pub user_id: UserId,
    pub exp: u64,
    pub provider: String,
}
