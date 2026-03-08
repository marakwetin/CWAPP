use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Connection pool error: {0}")]
    Pool(#[from] r2d2::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Template error: {0}")]
    Template(#[from] tera::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Db(_) | AppError::Pool(_) | AppError::Template(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorBody {
            error: self.to_string(),
        })
    }
}
