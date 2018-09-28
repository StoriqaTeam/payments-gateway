#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTClaims {
    user_id: usize,
    exp: u64,
    provider: String,
}
