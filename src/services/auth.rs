use std::sync::Arc;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use jsonwebtoken::{decode, Algorithm, Validation};
use secp256k1::{Message, PublicKey, Secp256k1, Signature};
use validator::{ValidationError, ValidationErrors};

use super::error::*;
use super::ServiceFuture;
use models::*;
use prelude::*;
use repos::{DbExecutor, DevicesRepo};

pub trait AuthService: Send + Sync + 'static {
    fn get_jwt_auth(&self, token: StoriqaJWT) -> ServiceFuture<Auth>;
    fn authenticate(&self, info: AuthInfo, user_id: UserId) -> ServiceFuture<()>;
    fn get_exp(&self, token: StoriqaJWT) -> ServiceFuture<u64>;
}

pub struct AuthServiceImpl<E: DbExecutor> {
    jwt_public_key: Vec<u8>,
    jwt_valid_secs: usize,
    devices_repo: Arc<dyn DevicesRepo>,
    db_executor: E,
}

impl<E: DbExecutor> AuthServiceImpl<E> {
    pub fn new(jwt_public_key: Vec<u8>, jwt_valid_secs: usize, devices_repo: Arc<dyn DevicesRepo>, db_executor: E) -> Self {
        Self {
            jwt_public_key,
            jwt_valid_secs,
            devices_repo,
            db_executor,
        }
    }
}

impl<E: DbExecutor> AuthService for AuthServiceImpl<E> {
    fn get_jwt_auth(&self, token: StoriqaJWT) -> ServiceFuture<Auth> {
        let validation = Validation {
            leeway: self.jwt_valid_secs as i64,
            ..Validation::new(Algorithm::RS256)
        };
        let token_clone = token.clone();
        Box::new(
            decode::<JWTClaims>(token_clone.inner(), &self.jwt_public_key, &validation)
                .map_err(ectx!(ErrorContext::JsonWebToken, ErrorKind::Unauthorized => token_clone.inner()))
                .map(move |t| Auth {
                    user_id: t.claims.user_id,
                    token: StoriqaJWT::new(token.inner().to_string()),
                }).into_future(),
        )
    }
    fn authenticate(&self, info: AuthInfo, user_id: UserId) -> ServiceFuture<()> {
        let devices_repo = self.devices_repo.clone();
        let db_executor = self.db_executor.clone();
        Box::new(db_executor
                .execute(move || {
                    let device_id = info.device_id.clone();
                    let device = devices_repo
                        .get(device_id, user_id)
                        .map_err(ectx!(try convert => user_id, device_id))?;
                    if let Some(device) = device {
                        let info_timestamp = info.timestamp;
                        if info_timestamp < device.last_timestamp {
                            return Err(ectx!(err ErrorContext::WrongTimestamp, ErrorKind::Unauthorized => info_timestamp));
                        }
                        let hasher = Sha256::new();
                        hasher.input_str(&format!("{}{}",info.timestamp, info.device_id));
                        let mut bytes = [0; 32];
                        let hex = hasher.result(&mut bytes);
                        let message = Message::from_slice(&bytes).expect("32 bytes");
                        let secp = Secp256k1::new();
                        let public_key = PublicKey::from_slice(secp, device.public_key)?;
                        let sig = Signature::from_der(&info.sign);
                        secp.verify(&message, &sig, &public_key)
                        .map_err(ectx!(ErrorContext::Sign, ErrorKind::Unauthorized))?;
                        } else {
                        let mut errors = ValidationErrors::new();
                        let mut error = ValidationError::new("exists");
                        error.add_param("message".into(), &"device not exists".to_string());
                        error.add_param("details".into(), &"no details".to_string());
                        errors.add("device", error);
                        let device_id = info.device_id.clone();
                        return Err(ectx!(err ErrorContext::DeviceNotExists, ErrorKind::InvalidInput(serde_json::to_value(&errors).unwrap_or_default()) => user_id, device_id));
                    }
                    Ok(())
                }))
    }
    fn get_exp(&self, token: StoriqaJWT) -> ServiceFuture<u64> {
        let token_clone = token.clone();
        let validation = Validation {
            leeway: self.jwt_valid_secs as i64,
            ..Validation::new(Algorithm::RS256)
        };
        Box::new(
            decode::<JWTClaims>(token_clone.inner(), &self.jwt_public_key, &validation)
                .map_err(ectx!(ErrorContext::JsonWebToken, ErrorKind::Unauthorized => token_clone.inner()))
                .map(move |t| t.claims.exp)
                .into_future(),
        )
    }
}
