// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::anyhow;
use async_trait::async_trait;
use identity_iota::core::Timestamp;
use tokio_pg_mapper::FromTokioPostgresRow;
use crate::{models::download_request::DownloadRequest, errors::ConnectorError};
use deadpool_postgres::Client as PostgresClient;


#[async_trait]
pub trait ChallengesExt {
    async fn insert_challenge(&self, download_request: &DownloadRequest) -> Result<DownloadRequest, ConnectorError>;
    async fn get_challenge(&self, did: &String, nonce: &String, timestamp: Timestamp) -> Result<DownloadRequest, ConnectorError>;
    async fn remove_challenge(&self, nonce: &String) ->  Result<(), ConnectorError>;
    async fn cleanup_challenge(&self) -> Result<(), anyhow::Error>;
}

#[async_trait]
impl ChallengesExt for PostgresClient {

    async fn get_challenge (
        &self, 
        did: &String,
        nonce: &String,
        timestamp: Timestamp
    ) -> Result<DownloadRequest, ConnectorError> {

        let _stmt = include_str!("../../sql/download_request_get.sql");
        let _stmt = _stmt.replace("$table_fields", &DownloadRequest::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;

        match self
        .query_one(&stmt, &[did, nonce, &timestamp.to_rfc3339()])
        .await{
            Ok(row) => DownloadRequest::from_row_ref(&row).map_err(|e| ConnectorError::from(e)),
            Err(_) =>  Err(ConnectorError::RowNotFound),
        }
    
    }

    async fn insert_challenge (
        &self, 
        download_request: &DownloadRequest,
    ) -> Result<DownloadRequest, ConnectorError>{
        let _stmt = include_str!("../../sql/download_request_insert.sql");
        let _stmt = _stmt.replace("$table_fields", &DownloadRequest::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;

        self.query(
            &stmt,
            &[  
                &download_request.nonce,
                &download_request.requester_did,
                &download_request.expiration, //.to_rfc3339(),
            ],
        )
        .await?
        .iter()
        .map(|row| DownloadRequest::from_row_ref(row).unwrap())
        .collect::<Vec<DownloadRequest>>()
        .pop()
        .ok_or(ConnectorError::RowNotFound) // more applicable for SELECTs
    }

    async fn remove_challenge (
        &self, 
        nonce: &String
    ) ->  Result<(), ConnectorError> {
        let _stmt = include_str!("../../sql/download_request_remove.sql");
        let stmt = self.prepare(&_stmt).await?;

        self.query(&stmt, &[nonce]).await?;
        Ok(())
    }

    async fn cleanup_challenge(&self) -> Result<(), anyhow::Error>{
        let _stmt = include_str!("../../sql/download_request_cleanup.sql");
        let stmt = self.prepare(&_stmt).await?;

        self.query(&stmt, &[&Timestamp::now_utc().to_rfc3339()]).await
            .map_err(|e| anyhow!("SQL Query delete failed: {}", e.to_string()))?;

        Ok(())
    }

}