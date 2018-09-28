#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    email: String,
    first_name: String,
    last_name: String,
    phone: Option<String>,
}
