use std::fmt::{self, Display};
use std::str::FromStr;

use diesel::sql_types::Uuid as SqlUuid;
use uuid::{self, Uuid};

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, Clone, Copy, PartialEq)]
#[sql_type = "SqlUuid"]
pub struct ExchangeId(Uuid);
derive_newtype_sql!(exchange_id, SqlUuid, ExchangeId, ExchangeId);

impl ExchangeId {
    pub fn new(id: Uuid) -> Self {
        ExchangeId(id)
    }

    pub fn inner(&self) -> &Uuid {
        &self.0
    }

    pub fn generate() -> Self {
        ExchangeId(Uuid::new_v4())
    }
}

impl FromStr for ExchangeId {
    type Err = uuid::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = Uuid::parse_str(s)?;
        Ok(ExchangeId::new(id))
    }
}

impl Display for ExchangeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{}", self.0.hyphenated()))
    }
}
