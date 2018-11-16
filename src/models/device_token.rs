use chrono::NaiveDateTime;

use validator::Validate;

use models::*;
use schema::devices_tokens;

#[derive(Debug, Queryable, Clone)]
pub struct DeviceToken {
    pub id: DeviceConfirmToken,
    pub device_id: DeviceId,
    pub device_os: String,
    pub user_id: UserId,
    pub public_key: DevicePublicKey,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for DeviceToken {
    fn default() -> Self {
        Self {
            id: DeviceConfirmToken::generate(),
            device_id: DeviceId::generate(),
            device_os: String::default(),
            user_id: UserId::generate(),
            public_key: DevicePublicKey::generate(),
            created_at: ::chrono::Utc::now().naive_utc(),
            updated_at: ::chrono::Utc::now().naive_utc(),
        }
    }
}

impl From<NewDeviceToken> for DeviceToken {
    fn from(new_device: NewDeviceToken) -> Self {
        Self {
            id: new_device.id,
            device_id: new_device.device_id,
            device_os: new_device.device_os,
            user_id: new_device.user_id,
            public_key: new_device.public_key,
            ..Default::default()
        }
    }
}

#[derive(Debug, Insertable, Validate, Clone)]
#[table_name = "devices_tokens"]
pub struct NewDeviceToken {
    pub id: DeviceConfirmToken,
    pub device_id: DeviceId,
    pub device_os: String,
    pub user_id: UserId,
    pub public_key: DevicePublicKey,
}

impl Default for NewDeviceToken {
    fn default() -> Self {
        Self {
            id: DeviceConfirmToken::generate(),
            device_id: DeviceId::generate(),
            device_os: String::default(),
            user_id: UserId::generate(),
            public_key: DevicePublicKey::generate(),
        }
    }
}

impl NewDeviceToken {
    pub fn new(device_id: DeviceId, device_os: String, user_id: UserId, public_key: DevicePublicKey) -> Self {
        Self {
            id: DeviceConfirmToken::generate(),
            device_id,
            device_os,
            user_id,
            public_key,
        }
    }
}
