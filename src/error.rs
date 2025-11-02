use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use reqwest::Error as ReqwestError;
use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("classroom not found")]
    ClassroomNotFound,
    #[error("user not found")]
    UserNotFound,
    #[error("invalid request: {0}")]
    BadRequest(String),
    #[error("database error: {0}")]
    Database(#[from] DbErr),
    #[error("external service error: {0}")]
    External(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::ClassroomNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Database(err) => {
                let status = match err {
                    DbErr::RecordNotFound(_) => StatusCode::NOT_FOUND,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (status, "internal server error".to_string())
            }
            AppError::External(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
        };

        let body = Json(ErrorResponse { message });
        (status, body).into_response()
    }
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        Self::BadRequest(value.to_owned())
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self::BadRequest(value)
    }
}

impl From<ReqwestError> for AppError {
    fn from(value: ReqwestError) -> Self {
        Self::External(value.to_string())
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}
