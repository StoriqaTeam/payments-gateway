use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};

use regex::Regex;
use serde::{Serialize, Serializer};
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Deserialize, Clone)]
pub struct Password(String);

const PASSWORD_MASK: &str = "********";

lazy_static! {
    static ref REG_CONTAINS_NUMBERS: Regex = Regex::new(r#"\d"#).unwrap();
}

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

pub fn validate_password_len(password: &Password) -> Result<(), ValidationError> {
    let password_len = password.0.len();
    if password_len < 8 || password_len > 30 {
        Err(ValidationError {
            code: Cow::from("len"),
            message: Some(Cow::from("Password should be between 8 and 30 symbols")),
            params: HashMap::new(),
        })
    } else {
        Ok(())
    }
}
pub fn validate_password_lower_case(password: &Password) -> Result<(), ValidationError> {
    if password.0 == password.0.to_lowercase() {
        Err(ValidationError {
            code: Cow::from("upper case"),
            message: Some(Cow::from("Password should contain at least one upper case character")),
            params: HashMap::new(),
        })
    } else {
        Ok(())
    }
}
pub fn validate_password_numbers(password: &Password) -> Result<(), ValidationError> {
    if !REG_CONTAINS_NUMBERS.is_match(&password.0) {
        Err(ValidationError {
            code: Cow::from("numbers"),
            message: Some(Cow::from("Password should contain at least one number")),
            params: HashMap::new(),
        })
    } else {
        Ok(())
    }
}

impl Validate for Password {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let password_len = self.0.len();
        let mut errors = ValidationErrors::new();
        if password_len < 8 || password_len > 30 {
            let error = ValidationError {
                code: Cow::from("len"),
                message: Some(Cow::from("Password should be between 8 and 30 symbols")),
                params: HashMap::new(),
            };
            errors.add("password", error);
        }
        if self.0 == self.0.to_lowercase() {
            let error = ValidationError {
                code: Cow::from("upper case"),
                message: Some(Cow::from("Password should contain at least one upper case character")),
                params: HashMap::new(),
            };
            errors.add("password", error);
        }
        if !REG_CONTAINS_NUMBERS.is_match(&self.0) {
            let error = ValidationError {
                code: Cow::from("numbers"),
                message: Some(Cow::from("Password should contain at least one number")),
                params: HashMap::new(),
            };
            errors.add("password", error);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Password {
    pub fn inner(&self) -> &str {
        &self.0
    }
}
