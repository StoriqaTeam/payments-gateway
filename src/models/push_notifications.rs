use api::responses::*;
use models::*;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PushNotifications {
    pub device_id: DeviceId,
    pub device_os: String,
    pub transaction: TransactionsResponse,
}

impl PushNotifications {
    pub fn new(device_id: DeviceId, device_os: String, transaction: TransactionsResponse) -> Self {
        Self {
            device_id,
            device_os,
            transaction,
        }
    }
}
