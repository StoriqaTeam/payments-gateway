use std::fmt::{self, Display};
use std::num::ParseIntError;
use std::str::FromStr;

use diesel::sql_types::Int4 as SqlInt4;

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, Clone, Copy, Default, PartialEq)]
#[sql_type = "SqlInt4"]
pub struct UserId(i32);
derive_newtype_sql!(user_id, SqlInt4, UserId, UserId);

impl UserId {
    pub fn new(id: i32) -> Self {
        UserId(id)
    }
    pub fn inner(&self) -> &i32 {
        &self.0
    }
    pub fn generate() -> Self {
        UserId(i32::default())
    }
}

impl FromStr for UserId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse()?;
        Ok(UserId::new(id))
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{}", self.0,))
    }
}
