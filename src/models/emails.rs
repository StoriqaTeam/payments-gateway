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
    pub url: String,
    pub token: DeviceConfirmToken,
    pub device_id: DeviceId,
    pub user: UserDB,
}

impl DeviceAddEmail {
    pub fn new(url: String, token: DeviceConfirmToken, device_id: DeviceId, user: UserDB) -> Self {
        Self {
            url,
            token,
            device_id,
            user,
        }
    }
}

impl Email {
    pub fn new(to: String, subject: String, text: String) -> Self {
        Self { to, subject, text }
    }
}
