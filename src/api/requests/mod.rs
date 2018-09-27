use models::Password;

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum DeviceType {
    Ios,
    Android,
    Web,
    Other,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostSessionsRequest {
    pub email: String,
    pub password: Password,
    pub device_type: DeviceType,
    pub device_os: Option<String>,
    pub device_id: Option<String>,
}
