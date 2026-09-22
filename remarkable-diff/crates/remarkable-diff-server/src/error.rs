//! Error types for the server

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("Version not found: {0}")]
    VersionNotFound(String),
    
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Merge error: {0}")]
    MergeError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::DocumentNotFound(_) | AppError::VersionNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            AppError::InvalidFormat(_) | AppError::ParseError(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            AppError::MergeError(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }
            AppError::Internal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };
        
        let body = Json(json!({
            "error": message,
            "code": status.as_u16()
        }));
        
        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
