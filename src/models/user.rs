use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[validate(email(message = "Invalid email format"))]
    email: String,
    #[validate(length(min = "1", message = "First name must not be empty"))]
    pub first_name: String,
    #[validate(length(min = "1", message = "Last name must not be empty"))]
    pub last_name: String,
    #[validate(phone(message = "Invalid email format"))]
    phone: Option<String>,
}
