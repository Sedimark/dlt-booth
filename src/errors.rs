// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_web::{HttpResponse, ResponseError, http::header::ContentType};
use deadpool_postgres::PoolError;
use reqwest::StatusCode;
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum ConnectorError {

    #[error("Credential Missing")]
    CredentialMissing,

    // IOTA Errors
    #[error("Iota Client Error")]
    IotaClientError(#[from] iota_sdk::client::Error),
    #[error("Resolve Error")]
    ResolveError(#[from] identity_iota::iota::Error),
    #[error("Did Error")]
    DidError(#[from] identity_iota::did::Error),
    #[error("Jwk error")]
    JwkError(#[from]identity_iota::storage::JwkStorageDocumentError),
    #[error("Credential Error")]
    CredentialError(#[from] identity_iota::credential::Error),
    #[error("Verification method Error")]
    VerificationMethodError(#[from] identity_iota::verification::Error),
    
    #[error("Persist File Error")]
    PersistFileError,
    #[error("Creating Folder Error")]
    CreatingUploadFolder(#[from] std::io::Error),
    #[error("File Name Missing Error")]
    FileNameMissing,
    #[error("File Upload Error")]
    FileUploadError,
    #[error("Ipfs Upload Error")]
    IpfsUploadError(#[from] ipfs_api_backend_actix::Error),
    #[error("serde_json Error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Missing Id Error")]
    IdMissing,

    // Database Errors
    #[error("Row not found")]   
    RowNotFound,
    #[error("tokio_postgres error")]
    TokioPostgresError(#[from] tokio_postgres::error::Error),
    #[error("tokio_pg_mapper error")]
    TokioPostgresMapperError(#[from] tokio_pg_mapper::Error),
    #[error("Pool error")]
    PoolError(#[from] PoolError),
}   

impl ResponseError for ConnectorError {

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .json(json!({
                "error": self.to_string()
            }))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ConnectorError::IotaClientError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::RowNotFound => StatusCode::NOT_FOUND,
            ConnectorError::TokioPostgresError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::TokioPostgresMapperError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::PoolError(_) => StatusCode::FORBIDDEN,
            ConnectorError::ResolveError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::DidError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::JwkError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::CredentialMissing => StatusCode::BAD_REQUEST,
            ConnectorError::CredentialError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::PersistFileError => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::CreatingUploadFolder(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::FileNameMissing => StatusCode::BAD_REQUEST,
            ConnectorError::FileUploadError => StatusCode::BAD_REQUEST,
            ConnectorError::IpfsUploadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::SerdeJsonError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::IdMissing => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::VerificationMethodError(_) => StatusCode::INTERNAL_SERVER_ERROR,

        }
    }
}