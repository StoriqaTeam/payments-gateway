use std::fmt::{Debug, Error, Formatter};

use serde::{Serialize, Serializer};
use validator::{Validate, ValidationErrors};

#[derive(Deserialize, Clone)]
pub struct Password(String);

const PASSWORD_MASK: &str = "********";

impl Debug for Password {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(PASSWORD_MASK)
    }
}

impl Serialize for Password {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(PASSWORD_MASK)
    }
}

impl Validate for Password {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let password_len = self.0.len();
        if password_len < 8 || password_len > 30 {
            let error = validation_errors!({
                "password": ["len" => "Password should be between 8 and 30 symbols"]
            });
            Err(error)
        } else {
            Ok(())
        }
    }
}

impl Password {
    pub fn inner(&self) -> &str {
        &self.0
    }
}
