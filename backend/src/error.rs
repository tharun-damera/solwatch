use std::{env::VarError, io::Error as IoError};

use axum::{Json, http::StatusCode, response::IntoResponse};
use mongodb::bson::de::Error as MongoDeserializeError;
use mongodb::bson::ser::Error as MongoSerializeError;
use mongodb::error::Error as MongoError;
use serde::Serialize;
use serde_json::Error as SerdeJsonError;
use solana_client::client_error::ClientError;
use solana_sdk::{pubkey::ParsePubkeyError, signature::ParseSignatureError};
use thiserror::Error;
use tracing::{error, instrument};

// Create an AppError using thiserror that handles almost all errors
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad Request - {0}")]
    BadRequest(String),

    #[error("{0} Not Found")]
    NotFound(String),

    #[error("Internal Error - {0}")]
    Internal(String),

    #[error("Database Error - {0}")]
    Database(String),

    #[error("Solana Error - {0}")]
    Solana(String),
}

// Custom Error Response
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

// Send appropriate state code and the error message in the response when an error occurs in a handler
impl IntoResponse for AppError {
    #[instrument(skip_all)]
    fn into_response(self) -> axum::response::Response {
        let (status_code, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Solana(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        error!(?message);

        let body = Json(ErrorResponse { error: message });
        (status_code, body).into_response()
    }
}

// Map the MongoError to the Database variant of the AppError
impl From<MongoError> for AppError {
    fn from(e: MongoError) -> Self {
        AppError::Database(e.to_string())
    }
}
// Map the MongoSerializeError to the Database variant of the AppError
impl From<MongoSerializeError> for AppError {
    fn from(e: MongoSerializeError) -> Self {
        AppError::Database(e.to_string())
    }
}
// Map the MongoDeserializeError to the Database variant of the AppError
impl From<MongoDeserializeError> for AppError {
    fn from(e: MongoDeserializeError) -> Self {
        AppError::Database(e.to_string())
    }
}

// Map the std::io::Error to the Internal variant of the AppError
impl From<IoError> for AppError {
    fn from(e: IoError) -> Self {
        AppError::Internal(e.to_string())
    }
}

// Map the std::env::VarError to the Internal variant of the AppError
impl From<VarError> for AppError {
    fn from(e: VarError) -> Self {
        AppError::Internal(e.to_string())
    }
}

// Map the serde_json::Error to the Internal variant of the AppError
impl From<SerdeJsonError> for AppError {
    fn from(e: SerdeJsonError) -> Self {
        AppError::Internal(e.to_string())
    }
}

// Map the Solana ParsePubkeyError to the Solana variant of the AppError
impl From<ParsePubkeyError> for AppError {
    fn from(_: ParsePubkeyError) -> Self {
        AppError::Solana("Invalid Address".to_string())
    }
}

// Map the Solana ClientError to the Solana variant of the AppError
impl From<ClientError> for AppError {
    fn from(e: ClientError) -> Self {
        AppError::Solana(e.to_string())
    }
}

// Map the Solana ParseSignatureError to Solana
impl From<ParseSignatureError> for AppError {
    fn from(e: ParseSignatureError) -> Self {
        AppError::Solana(e.to_string())
    }
}
