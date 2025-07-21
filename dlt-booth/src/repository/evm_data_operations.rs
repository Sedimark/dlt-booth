// SPDX-FileCopyrightText: 2025 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::str::FromStr;

use alloy::primitives::Address;
use async_trait::async_trait;
use deadpool_postgres::Client as PostgresClient;


use crate::errors::ConnectorError;

#[async_trait]
pub trait EvmAddressesExt {
    async fn insert_address(&self, name: &str, address: &str) -> Result<(), ConnectorError>;
    async fn delete_all_addresses(&self) -> Result<(), ConnectorError>;
    async fn get_address(&self, name: &str) -> Result<Address, ConnectorError>;
}

#[async_trait]
impl EvmAddressesExt for PostgresClient {
    async fn insert_address(&self, name: &str, address: &str) -> Result<(), ConnectorError>
    {
        let stmt = include_str!("../../sql/evm_addresses_insert.sql");
        let stmt = self.prepare(stmt).await?;
        self.query(
            &stmt,
            &[
                &name,
                &address.to_string()
            ]
        )
        .await?;
        Ok(())
    }

    async fn delete_all_addresses(&self) -> Result<(), ConnectorError>
    {
        let stmt = include_str!("../../sql/evm_addresses_delete.sql");
        let stmt = self.prepare(stmt).await?;

        self.query(&stmt, &[])
        .await?;

        Ok(())
    }
    async fn get_address(&self, name: &str) -> Result<Address, ConnectorError>
    {
        let stmt = include_str!("../../sql/evm_addresses_get.sql");
        let stmt = self.prepare(stmt).await?;

        let address_str: String = self.query_one(&stmt, &[&name])
            .await?
            .get("evm_address");
        let address = Address::from_str(&address_str)?;
        Ok(address)
    }
}