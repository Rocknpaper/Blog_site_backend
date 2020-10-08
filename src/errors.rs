use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppErrorType {
    DatabaseError,
    NotFoundError,
    JWtTokenError,
    JWTParsingError,
    FileUploadError,
    InavlidId,
}

#[derive(Debug)]
pub struct AppError {
    pub message: Option<String>,
    pub cause: Option<String>,
    pub error_type: AppErrorType,
}

impl AppError {
    fn message(&self) -> String {
        match &*self {
            AppError {
                message: Some(message),
                cause: _,
                error_type: _,
            } => message.clone(),
            AppError {
                message: None,
                cause: _,
                error_type: _,
            } => "Unexpected Error".to_string(),
        }
    }
    fn cause(&self) -> String {
        match &*self {
            AppError {
                message: _,
                cause: Some(message),
                error_type: _,
            } => message.clone(),
            AppError {
                message: _,
                cause: None,
                error_type: _,
            } => "Unexpected Error".to_string(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize)]
pub struct AppErrorResponse {
    pub error: String,
    pub cause: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            AppErrorType::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::NotFoundError => StatusCode::NOT_FOUND,
            AppErrorType::JWtTokenError => StatusCode::UNAUTHORIZED,
            AppErrorType::FileUploadError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::JWTParsingError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::InavlidId => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            error: self.message(),
            cause: self.cause(),
        })
    }
}
