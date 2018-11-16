use models::*;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Email {
    pub to: String,
    pub subject: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceAddEmail {
    pub to: String,
    pub base_url: String,
    pub token: DeviceConfirmToken,
    pub device_id: DeviceId,
}

impl DeviceAddEmail {
    pub fn new(to: String, base_url: String, token: DeviceConfirmToken, device_id: DeviceId) -> Self {
        Self {
            to,
            base_url,
            token,
            device_id,
        }
    }
}

impl From<DeviceAddEmail> for Email {
    fn from(email: DeviceAddEmail) -> Self {
        Self {
                to : email.to,
                subject : format!("New device will be added to your wallet"),
                text : format!(
                    "Please, follow <a href=\"{}?token={}\">this link</a> to add new device with number `{}` to your account in Storiqa wallet.",
                    email.base_url, email.token, email.device_id
                ),
            }
    }
}
