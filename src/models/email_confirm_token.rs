use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailConfirmToken(String);

impl Display for EmailConfirmToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for EmailConfirmToken {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(EmailConfirmToken(s.into()))
    }
}
