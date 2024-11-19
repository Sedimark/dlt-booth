// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use actix_web::{HttpResponse, ResponseError, http::header::ContentType};
use deadpool_postgres::PoolError;
use hex::FromHexError;
use reqwest::StatusCode;
use serde_json::json;
use tokio::sync::TryLockError;
use std::marker::{Send, Sync};

unsafe impl Send for ConnectorError{}
unsafe impl Sync for ConnectorError{}

#[derive(thiserror::Error, Debug)]
pub enum ConnectorError {
    #[error("Credential Missing")]
    CredentialMissing,
    #[error("Challenge missing")]
    ChallengeMissing,

    // IOTA Errors
    #[error("Iota Client Error")]
    IotaClientError(#[from] iota_sdk::client::Error),
    #[error("Resolve Error")]
    ResolveError(#[from] identity_iota::iota::Error),
    #[error("Did Error")]
    DidError(#[from] identity_iota::did::Error),
    #[error("Jwk error")]
    JwkError(#[from] identity_iota::storage::JwkStorageDocumentError),
    #[error("Credential Error")]
    CredentialError(#[from] identity_iota::credential::Error),
    #[error("Verification method Error")]
    VerificationMethodError(#[from] identity_iota::verification::Error),
    #[error("Jwt Verification Error")]
    JwtValidationError(#[from] identity_iota::credential::JwtValidationError),
    #[error("Wallet unavailable with error: {0}")]
    WalletError(String),

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
    #[error("Pending challenge Error")]
    ChallengePendingError,
    #[error("Verification method for ethereum address verification not found")]
    EthMethodNotFound,
    #[error("Verification method type is not EcdsaSecp256k1RecoveryMethod2020")]
    InvalidVerificationMethodType,
    #[error("Nft address of the asset is missing")]
    NftAddressMissing,
    #[error("Address recovery error")]
    AddressRecoveryError,
    #[error("String to ethers::types::Bytes error")]
    StringToBytesError,
    #[error("Contract error")]
    ContractError,
    #[error("Unexpected signer error")]
    SignerError(#[from] alloy::signers::Error),

    // Database Errors
    #[error("Row not found")]   
    RowNotFound,
    #[error("tokio_postgres error")]
    TokioPostgresError(#[from] tokio_postgres::error::Error),
    #[error("tokio_pg_mapper error")]
    TokioPostgresMapperError(#[from] tokio_pg_mapper::Error),
    #[error("Pool error")]
    PoolError(#[from] PoolError),
    #[error("Middleware error: {0}")]
    MiddlewareError(String),

    #[error("Other error: {0}")]
    OtherError(String),
    #[error("Resource cannot be accessed")]
    ResourceError(#[from] TryLockError),
    #[error("Cannot convert from hex")]
    ConversionError(#[from] FromHexError)
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
            ConnectorError::JwtValidationError(_) => StatusCode::INTERNAL_SERVER_ERROR, // Update the match arm
            ConnectorError::ChallengePendingError => StatusCode::BAD_REQUEST,
            ConnectorError::MiddlewareError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::EthMethodNotFound => StatusCode::BAD_REQUEST,
            ConnectorError::InvalidVerificationMethodType => StatusCode::BAD_REQUEST,
            ConnectorError::NftAddressMissing => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::AddressRecoveryError => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::StringToBytesError => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::ContractError => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::OtherError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::ChallengeMissing => StatusCode::BAD_REQUEST,
            ConnectorError::WalletError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::ResourceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::SignerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::ConversionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}