// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use serde::{Serialize, Deserialize};
use tokio_pg_mapper_derive::PostgresMapper;


#[derive(Debug, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "assets")]
pub struct Asset {
    nft_address: String, 	
    cid: String,
    alias: String,
    asset_path: String,
    offering_path: String,
    asset_hash: String,
    offering_hash: String,
    sign: String,
	publisher: String,
}