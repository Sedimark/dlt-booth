// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Serialize, Deserialize};
use tokio_pg_mapper_derive::PostgresMapper;
// use tokio_postgres::types::Timestamp;

#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "download_requests")]
pub struct DownloadRequest {
    pub nonce: String,
    // pub asset_id: String,
    pub requester_did: String,
    pub expiration: String
}