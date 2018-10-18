use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoriqaJWT(String);

impl StoriqaJWT {
    pub fn new(token: String) -> Self {
        StoriqaJWT(token)
    }
}

impl StoriqaJWT {
    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl Default for StoriqaJWT {
    fn default() -> Self {
        StoriqaJWT(Uuid::new_v4().to_string())
    }
}
