use std::collections::HashMap;

use super::auth::AuthService;
use super::error::*;
use super::ServiceFuture;
use models::*;
use prelude::*;

pub struct AuthServiceMock {
    auths: HashMap<StoriqaJWT, Auth>,
}

impl AuthServiceMock {
    pub fn new(allowed_tokens: Vec<(StoriqaJWT, UserId)>) -> Self {
        let mut auths = HashMap::new();
        for (token, user_id) in allowed_tokens {
            let auth = Auth {
                token: StoriqaJWT::default(),
                user_id,
            };
            auths.insert(token, auth);
        }
        AuthServiceMock { auths }
    }
}

impl AuthService for AuthServiceMock {
    fn authenticate(&self, token: StoriqaJWT) -> ServiceFuture<Auth> {
        Box::new(
            self.auths
                .get(&token)
                .map(|x| x.clone())
                .ok_or(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized))
                .into_future(),
        )
    }
    fn get_exp(&self, token: StoriqaJWT) -> ServiceFuture<u64> {
        Box::new(
            self.auths
                .get(&token)
                .map(|_| 0)
                .ok_or(ectx!(err ErrorContext::InvalidToken, ErrorKind::Unauthorized))
                .into_future(),
        )
    }
}
