// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Serialize, Deserialize};
use tokio_pg_mapper_derive::PostgresMapper;


#[derive(Debug, Serialize, Deserialize, Clone, PostgresMapper)]
#[pg_mapper(table = "identities")]
pub struct Identity {
    pub id: Option<i64>,
    pub eth_address: String,
    pub did: String,
    pub fragment: String,
    pub vcredential: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CredentialIssuedResponse {
    pub message: String,
    pub issuer_did: String,
    pub credential_id: iota_sdk::U256,
    pub credential_jwt: identity_iota::credential::Jwt
}