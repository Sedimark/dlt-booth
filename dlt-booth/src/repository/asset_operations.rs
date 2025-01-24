// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use async_trait::async_trait;
use tokio_pg_mapper::FromTokioPostgresRow;
use crate::{models::asset::Asset, errors::ConnectorError};
use deadpool_postgres::Client as PostgresClient;


#[async_trait]
pub trait AssetExt {
    async fn insert_asset_info(&self, asset: &Asset) -> Result<Asset, ConnectorError>;
    async fn get_asset_info_from_alias(&self, alias: &String) -> Result<Asset, ConnectorError>;
    async fn get_asset_info(&self, id: i64) -> Result<Asset, ConnectorError>;
    async fn get_aliases(&self, eth_address: &String) -> Result<Vec<String>, ConnectorError>;
    async fn set_nft_address(&self, eth_address: &String, credential: &String) -> Result<Asset, ConnectorError>;
}

#[async_trait]
impl AssetExt for PostgresClient {

    async fn insert_asset_info(&self, asset: &Asset) -> Result<Asset, ConnectorError> {
        log::info!("insert asset");
        let _stmt = include_str!("../../sql/asset_insert.sql");
        let _stmt = _stmt.replace("$table_fields", &Asset::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;

        self.query(
            &stmt,
            &[
                &asset.cid,
                &asset.alias,
                &asset.asset_path,
                &asset.offering_path,
                &asset.asset_hash,
                &asset.offering_hash,
                &asset.sign,
                &asset.publisher
            ],
        )
        .await?
        .iter()
        .map(|row| Asset::from_row_ref(row).unwrap())
        .collect::<Vec<Asset>>()
        .pop()
        .ok_or(ConnectorError::RowNotFound) // more applicable for SELECTs
    }

    async fn get_asset_info_from_alias(&self, alias: &String) -> Result<Asset, ConnectorError> {
        log::info!("get asset");
        let _stmt = include_str!("../../sql/asset_get_with_alias.sql");
        let _stmt = _stmt.replace("$table_fields", &Asset::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;
    
        match self
        .query_one(&stmt, &[&alias])
        .await {
            Ok(row) =>{ Asset::from_row_ref(&row).map_err(|e| ConnectorError::from(e))},
            Err(_) =>  Err(ConnectorError::RowNotFound),
        }
        
    }
    
    async fn get_asset_info(&self, id: i64) -> Result<Asset, ConnectorError> {
        log::info!("get asset");
        let _stmt = include_str!("../../sql/asset_get.sql");
        let _stmt = _stmt.replace("$table_fields", &Asset::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;
    
        match self
        .query_one(&stmt, &[&id])
        .await {
            Ok(row) =>{ Asset::from_row_ref(&row).map_err(|e| ConnectorError::from(e))},
            Err(_) =>  Err(ConnectorError::RowNotFound),
        }
    }

    async fn get_aliases(&self, eth_address: &String) -> Result<Vec<String>, ConnectorError> {
        log::info!("get aliases");
        let stmt = include_str!("../../sql/asset_get_all_aliases.sql");
        let stmt = self.prepare(&stmt).await?;

        let results = self
        .query(&stmt, &[&eth_address])
        .await?
        .iter()
        .map(|row| row.get("alias") )
        .collect::<Vec<String>>();
        
        Ok(results)
    }

    async fn set_nft_address(
        &self, 
        alias: &String,
        nft_address: &String, 
    ) -> Result<Asset, ConnectorError> {
        log::info!("set nft address");
        let _stmt = include_str!("../../sql/asset_update_nft_addr.sql");
        let _stmt = _stmt.replace("$table_fields", &Asset::sql_table_fields());
        let stmt = self.prepare(&_stmt).await?;

        match self.query_one(&stmt, &[&nft_address, &alias])
        .await {
            Ok(row) => Asset::from_row_ref(&row).map_err(|e| ConnectorError::from(e)),
            Err(_) =>  Err(ConnectorError::RowNotFound),
        }
    }
    
}