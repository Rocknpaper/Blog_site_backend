use bcrypt::{hash, verify};

use crate::errors::{AppError, AppErrorType};

pub struct CryptoService ;

impl CryptoService {
    pub async fn hash_password(password: String) -> Result<String, AppError> {
        let hashed_password = hash(password, 12).map_err(|err| AppError {
            cause: Some(err.to_string()),
            message: None,
            error_type: AppErrorType::HashingError,
        })?;

        Ok(hashed_password)
    }

    pub async fn verify_hash(hashed_password: String, password: String) -> Result<bool, AppError> {
        let verify = verify(password, hashed_password.as_str()).map_err(|err| AppError {
            cause: Some(err.to_string()),
            message: None,
            error_type: AppErrorType::HashingError,
        })?;
        Ok(verify)
    }
}

