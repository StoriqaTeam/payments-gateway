use std::fmt;
use std::fmt::Display;

use diesel::sql_types::VarChar;
use uuid::Uuid;

#[derive(Serialize, Deserialize, FromSqlRow, AsExpression, Clone, Default, PartialEq, Eq, Hash, Debug)]
#[sql_type = "VarChar"]
pub struct DeviceId(String);
derive_newtype_sql!(device_id, VarChar, DeviceId, DeviceId);

impl Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl DeviceId {
    pub fn generate() -> Self {
        DeviceId(Uuid::new_v4().to_string())
    }
}
