use models::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    pub token: StoriqaJWT,
    pub user_id: UserId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthInfo {
    pub timestamp: i64,
    pub device_id: DeviceId,
    pub sign: String,
}

impl AuthInfo {
    pub fn new(timestamp: i64, device_id: DeviceId, sign: String) -> Self {
        Self {
            timestamp,
            device_id,
            sign,
        }
    }
}
