#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushNotifications {
    pub device_id: String,
}

impl Default for PushNotifications {
    fn default() -> Self {
        Self { device_id: String::default() }
    }
}
