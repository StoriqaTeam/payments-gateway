use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GetJWTResponse {
    pub data: Option<GetJWTByEmail>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GetJWTByEmail {
    pub getJWTByEmail: Token,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Token {
    pub token: String,
}
