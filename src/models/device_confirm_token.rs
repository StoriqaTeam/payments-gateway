use std::fmt::{self, Display};
use std::str::FromStr;

use diesel::sql_types::Uuid as SqlUuid;
use uuid::{self, Uuid};

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, Clone, Copy, PartialEq)]
#[sql_type = "SqlUuid"]
pub struct DeviceConfirmToken(Uuid);
derive_newtype_sql!(device_confirm_token, SqlUuid, DeviceConfirmToken, DeviceConfirmToken);

impl DeviceConfirmToken {
    pub fn new(id: Uuid) -> Self {
        DeviceConfirmToken(id)
    }

    pub fn inner(&self) -> &Uuid {
        &self.0
    }

    pub fn generate() -> Self {
        DeviceConfirmToken(Uuid::new_v4())
    }
}

impl FromStr for DeviceConfirmToken {
    type Err = uuid::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = Uuid::parse_str(s)?;
        Ok(DeviceConfirmToken::new(id))
    }
}

impl Display for DeviceConfirmToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{}", self.0.hyphenated()))
    }
}
