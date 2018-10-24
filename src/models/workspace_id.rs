use std::fmt::{self, Display};
use std::str::FromStr;

use diesel::sql_types::Uuid as SqlUuid;
use uuid::{ParseError, Uuid};

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, Clone, Copy, Default, PartialEq)]
#[sql_type = "SqlUuid"]
pub struct WorkspaceId(Uuid);
derive_newtype_sql!(user_id, SqlUuid, WorkspaceId, WorkspaceId);

impl WorkspaceId {
    pub fn new(id: Uuid) -> Self {
        WorkspaceId(id)
    }
    pub fn inner(&self) -> &Uuid {
        &self.0
    }
    pub fn generate() -> Self {
        WorkspaceId(Uuid::new_v4())
    }
}

impl FromStr for WorkspaceId {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = Uuid::parse_str(s)?;
        Ok(WorkspaceId::new(id))
    }
}

impl Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{}", self.0.hyphenated()))
    }
}
