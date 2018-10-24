use validator::Validate;

use models::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename(deserialize = "rawId"))]
    pub id: UserId,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = "1", message = "First name must not be empty"))]
    pub first_name: String,
    #[validate(length(min = "1", message = "Last name must not be empty"))]
    pub last_name: String,
    #[validate(
        custom = "validate_password_len",
        custom = "validate_password_lower_case",
        custom = "validate_password_numbers"
    )]
    pub password: Password,
    pub device_type: DeviceType,
}
