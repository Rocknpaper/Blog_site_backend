use chrono::Utc;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppErrorType};

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    fn new(sub: String, exp: usize) -> Self {
        Claims { sub, exp }
    }

    pub async fn encode_req(sub: &str) -> Result<String, AppError> {
        let time = Utc::now().timestamp() as usize + 86400 as usize;
        match encode(
            &Header::new(Algorithm::HS256),
            &Claims::new(sub.to_owned(), time),
            &EncodingKey::from_secret(b"hello"),
        ) {
            Ok(val) => Ok(val),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: Some("JWT Encoding Error".to_string()),
                error_type: AppErrorType::JWTParsingError,
            }),
        }
    }

    pub fn decode_req(token: &str) -> Result<TokenData<Claims>, AppError> {
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(b"hello"),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(val) => Ok(val),
            Err(_e) => Err(AppError {
                cause: Some(_e.to_string()),
                message: Some("JWT Decoding Error".to_string()),
                error_type: AppErrorType::JWTParsingError,
            }),
        }
    }
}
