// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_web::{HttpResponse, ResponseError, http::header::ContentType};
use deadpool_postgres::PoolError;
use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum ConnectorError {

    // IOTA Errors
    #[error("Iota Client Error")]
    IotaClientError(#[from] iota_sdk::client::Error),
    #[error("Resolve Error")]
    ResolveError(#[from] identity_iota::iota::Error),
    #[error("Did Error")]
    DidError(#[from] identity_iota::did::Error),
    #[error("Jwk error")]
    JwkError(#[from]identity_iota::storage::JwkStorageDocumentError),

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
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ConnectorError::IotaClientError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::RowNotFound => StatusCode::NOT_FOUND,
            ConnectorError::TokioPostgresError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::TokioPostgresMapperError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConnectorError::PoolError(_) => StatusCode::FORBIDDEN,
            ConnectorError::ResolveError(_) => todo!(),
            ConnectorError::DidError(_) => todo!(),
            ConnectorError::JwkError(_) => todo!(),

        }
    }
}