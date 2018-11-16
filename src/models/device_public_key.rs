use std::fmt;
use std::fmt::Display;

use diesel::sql_types::VarChar;
use uuid::Uuid;

#[derive(Deserialize, Serialize, FromSqlRow, AsExpression, Clone, Default, PartialEq, Eq, Hash, Debug)]
#[sql_type = "VarChar"]
pub struct DevicePublicKey(String);
derive_newtype_sql!(device_public_key, VarChar, DevicePublicKey, DevicePublicKey);

impl Display for DevicePublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl DevicePublicKey {
    pub fn generate() -> Self {
        DevicePublicKey(Uuid::new_v4().to_string())
    }

    pub fn inner(&self) -> String {
        self.0.clone()
    }
}
