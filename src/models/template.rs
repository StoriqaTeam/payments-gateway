use std::fmt::{self, Display};
use std::io::Write;

use chrono::NaiveDateTime;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::VarChar;

use schema::templates;

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, Clone, Copy, Eq, PartialEq, Hash)]
#[sql_type = "VarChar"]
#[serde(rename_all = "lowercase")]
pub enum TemplateName {
    AddDevice,
}

impl FromSql<VarChar, Pg> for TemplateName {
    fn from_sql(data: Option<&[u8]>) -> deserialize::Result<Self> {
        match data {
            Some(b"add_device") => Ok(TemplateName::AddDevice),
            Some(v) => Err(format!(
                "Unrecognized enum variant: {:?}",
                String::from_utf8(v.to_vec()).unwrap_or_else(|_| "Non - UTF8 value".to_string())
            ).to_string()
            .into()),
            None => Err("Unexpected null for non-null column".into()),
        }
    }
}

impl ToSql<VarChar, Pg> for TemplateName {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match self {
            TemplateName::AddDevice => out.write_all(b"add_device")?,
        };
        Ok(IsNull::No)
    }
}

impl Display for TemplateName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TemplateName::AddDevice => f.write_str("add_device"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Queryable, Insertable, Debug)]
#[table_name = "templates"]
pub struct Template {
    pub id: i32,
    pub name: TemplateName,
    pub data: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Insertable, Clone, Debug)]
#[table_name = "templates"]
pub struct NewTemplate {
    pub name: TemplateName,
    pub data: String,
}

impl Default for NewTemplate {
    fn default() -> Self {
        Self {
            name: TemplateName::AddDevice,
            data: String::default(),
        }
    }
}
