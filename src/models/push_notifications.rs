use api::responses::*;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushNotifications {
    pub device_id: String,
    pub transaction: TransactionsResponse,
}

impl PushNotifications {
    pub fn new(device_id: String, transaction: TransactionsResponse) -> Self {
        Self { device_id, transaction }
    }
}
