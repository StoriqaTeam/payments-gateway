use chrono::NaiveDateTime;

use validator::Validate;

use models::*;
use schema::devices;

#[derive(Debug, Queryable, Clone)]
pub struct Device {
    pub device_id: DeviceId,
    pub device_os: String,
    pub user_id: UserId,
    pub public_key: DevicePublicKey,
    pub last_timestamp: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            device_id: DeviceId::generate(),
            device_os: String::default(),
            user_id: UserId::generate(),
            public_key: DevicePublicKey::generate(),
            last_timestamp: 0,
            created_at: ::chrono::Utc::now().naive_utc(),
            updated_at: ::chrono::Utc::now().naive_utc(),
        }
    }
}

impl From<NewDevice> for Device {
    fn from(new_device: NewDevice) -> Self {
        Self {
            device_id: new_device.device_id,
            device_os: new_device.device_os,
            user_id: new_device.user_id,
            public_key: new_device.public_key,
            last_timestamp: 0,
            ..Default::default()
        }
    }
}

#[derive(Debug, Insertable, Validate, Clone)]
#[table_name = "devices"]
pub struct NewDevice {
    pub device_id: DeviceId,
    pub device_os: String,
    pub user_id: UserId,
    pub public_key: DevicePublicKey,
}

impl Default for NewDevice {
    fn default() -> Self {
        Self {
            device_id: DeviceId::generate(),
            device_os: String::default(),
            user_id: UserId::generate(),
            public_key: DevicePublicKey::generate(),
        }
    }
}

impl NewDevice {
    pub fn new(device_id: DeviceId, device_os: String, user_id: UserId, public_key: DevicePublicKey) -> Self {
        Self {
            device_id,
            device_os,
            user_id,
            public_key,
        }
    }
}
