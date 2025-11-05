use sqlx::{Error as SqlxError, migrate::MigrateError};
use std::{env::VarError, io::Error as IoError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database Error: {0}")]
    DatabaseError(String),

    #[error("Axum Error: {0}")]
    AxumError(#[from] axum::Error),

    #[error("Internal Error: {0}")]
    InternalError(String),
}

impl From<SqlxError> for AppError {
    fn from(e: SqlxError) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

impl From<MigrateError> for AppError {
    fn from(e: MigrateError) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

impl From<IoError> for AppError {
    fn from(e: IoError) -> Self {
        AppError::InternalError(e.to_string())
    }
}

impl From<VarError> for AppError {
    fn from(e: VarError) -> Self {
        AppError::InternalError(e.to_string())
    }
}
