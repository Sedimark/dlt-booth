use async_trait::async_trait;
use tokio_pg_mapper::FromTokioPostgresRow;
use crate::{models::identity::Identity, errors::ConnectorError};
use deadpool_postgres::Client as PostgresClient;


#[async_trait]
pub trait IdentityExt {
    async fn insert_identity(&self, identity: &Identity) -> Result<Identity, ConnectorError>;
    async fn get_identity_did(&self, eth_address: String) -> Result<Identity, ConnectorError>;
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

    async fn get_identity_did(&self, eth_address: String) -> Result<Identity, ConnectorError> {
        let _stmt = include_str!("../../sql/identity_get.sql");
        let _stmt = _stmt.replace("$table_fields", &Identity::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;
    
        match self
        .query_one(&stmt, &[&eth_address])
        .await{
            Ok(row) => Identity::from_row_ref(&row).map_err(|e| ConnectorError::from(e)),
            Err(_) =>  Err(ConnectorError::RowNotFound),
        }
    }
    
}