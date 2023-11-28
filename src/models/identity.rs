// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use serde::{Serialize, Deserialize};
use tokio_pg_mapper_derive::PostgresMapper;


#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "identities")]
pub struct Identity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub eth_address: String,
    pub did: String,
    pub fragment: String,
    pub vcredential: Option<String>,
}