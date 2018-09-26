pub struct StoriqaJWT(String);

impl StoriqaJWT {
    pub fn new(data: String) -> Self {
        StoriqaJWT(data)
    }
}
