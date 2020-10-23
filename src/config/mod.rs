use dotenv::dotenv;
use mongodb::{Client, Database};
use std::env::var;

use crate::errors::{AppError, AppErrorType};

pub mod crypto;
pub mod jwt;
pub mod s3_aws;

pub struct Config {
    pub host: String,
    pub port: String,
    pub mongodb_uri: String,
    pub db_name: String,
    // pub secret_key: String,
}

impl Config {
    pub fn from_env() -> Config {
        dotenv().ok();

        Config {
            host: var("host").unwrap(),
            port: var("port").unwrap(),
            mongodb_uri: var("mongodb_uri").unwrap(),
            // secret_key: var("SECRET_KEY").unwrap(),
            db_name: var("db_name").unwrap(),
        }
    }

    pub async fn get_db(&self) -> Result<Database, AppError> {
        Ok(Client::with_uri_str(&self.mongodb_uri)
            .await
            .map_err(|err| AppError {
                cause: Some(err.to_string()),
                message: None,
                error_type: AppErrorType::DatabaseError,
            })?
            .database(&self.db_name))
    }

    pub async fn crypto_services(&self) -> crypto::CryptoService {
        crypto::CryptoService
    }
}
