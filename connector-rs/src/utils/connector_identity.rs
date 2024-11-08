// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use deadpool_postgres::Pool;
use crate::{errors::ConnectorError, models::identity::Identity, repository::identity_operations::IdentityExt};
use super::iota::IotaState;

/// Create a new DID Document for the connector itself
pub async fn create_self_identity(db_pool: &Pool, iota_state: &IotaState) -> Result<Identity, ConnectorError>{
    log::info!("Create_identity for connector");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;

    let eth_address = iota_state.create_evm_address(Some(61),0).await?;

    // check if did already present
    let identity = pg_client.get_identity_with_eth_addr(&eth_address)
    .await;

    match identity{
        Ok(id) => Ok(id),
        Err(ConnectorError::RowNotFound) => {
            let (doc, fragment) = iota_state.create_did(Some(eth_address.as_str())).await?;
    
            let new_identity = Identity {
                id: None,
                eth_address: eth_address,
                did: doc.id().to_string(),
                fragment: fragment,
                vcredential: None,
            };
            pg_client.insert_identity(&new_identity).await
        }
        Err(e) => Err(e) 
    }
    
}