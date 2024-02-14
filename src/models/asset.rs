// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Serialize, Deserialize};
use tokio_pg_mapper_derive::PostgresMapper;


#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "assets")]
pub struct Asset {
    pub id: Option<i64>,
    pub nft_address: Option<String>,
    pub cid: String,
    pub alias: String,
    pub asset_path: String,
    pub offering_path: String,
    pub asset_hash: String,
    pub offering_hash: String,
    pub sign: String,
	pub publisher: i64,
}