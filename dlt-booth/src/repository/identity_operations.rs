// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use async_trait::async_trait;
use tokio_pg_mapper::FromTokioPostgresRow;
use crate::{models::identity::Identity, errors::ConnectorError};
use deadpool_postgres::Client as PostgresClient;


#[async_trait]
pub trait IdentityExt {
    async fn insert_identity(&self, identity: &Identity) -> Result<Identity, ConnectorError>;
    async fn get_identity_with_eth_addr(&self, eth_address: &String) -> Result<Identity, ConnectorError>;
    async fn get_identity(&self, id: i64) -> Result<Identity, ConnectorError>;
    async fn set_credential(&self, eth_address: &String, credential: &Option<String>) -> Result<Identity, ConnectorError>;
    async fn delete_credential(&self, did: &str) -> Result<(), ConnectorError>;
}

#[async_trait]
impl IdentityExt for PostgresClient {

    async fn insert_identity(&self, identity: &Identity) -> Result<Identity, ConnectorError>{
        log::info!("insert identity");
        let _stmt = include_str!("../../sql/identity_insert.sql");
        let _stmt = _stmt.replace("$table_fields", &Identity::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;

        self.query(
            &stmt,
            &[
                &identity.eth_address,
                &identity.did,
                &identity.fragment,
            ],
        )
        .await?
        .iter()
        .map(|row| Identity::from_row_ref(row).unwrap())
        .collect::<Vec<Identity>>()
        .pop()
        .ok_or(ConnectorError::RowNotFound) // more applicable for SELECTs
    }

    async fn get_identity_with_eth_addr(&self, eth_address: &String) -> Result<Identity, ConnectorError> {
        log::info!("get identity");
        let _stmt = include_str!("../../sql/identity_get_with_eth_addr.sql");
        let _stmt = _stmt.replace("$table_fields", &Identity::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;
    
        match self
        .query_one(&stmt, &[&eth_address])
        .await {
            Ok(row) =>{ log::info!("..{:?}", row); Identity::from_row_ref(&row).map_err(|e| ConnectorError::from(e))},
            Err(_) =>  Err(ConnectorError::RowNotFound),
        }
    }

    async fn get_identity(&self, id: i64) -> Result<Identity, ConnectorError> {
        log::info!("get identity");
        let _stmt = include_str!("../../sql/identity_get.sql");
        let _stmt = _stmt.replace("$table_fields", &Identity::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;
    
        match self
        .query_one(&stmt, &[&id])
        .await {
            Ok(row) => Identity::from_row_ref(&row).map_err(|e| ConnectorError::from(e)),
            Err(_) =>  Err(ConnectorError::RowNotFound),
        }
    }

    async fn set_credential(
        &self, 
        eth_address: &String,
        credential: &Option<String>,
    ) -> Result<Identity, ConnectorError> {
        log::info!("set credential");
        let _stmt = include_str!("../../sql/identity_update_vc.sql");
        let _stmt = _stmt.replace("$table_fields", &Identity::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;

        match self.query_one(&stmt, &[&credential,&eth_address])
        .await {
            Ok(row) => Identity::from_row_ref(&row).map_err(|e| ConnectorError::from(e)),
            Err(_) =>  Err(ConnectorError::RowNotFound),
        }

    }

    async fn delete_credential(&self, did: &str) -> Result<(), ConnectorError>{
        log::info!("set credential");
        let _stmt = include_str!("../../sql/identity_delete.sql");
        let stmt = self.prepare(&_stmt).await?;

        self.query(&stmt, &[&did.to_string()]).await?;

        Ok(())
    }
}