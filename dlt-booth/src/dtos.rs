// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityResponse {
    pub id: Option<i64>,
    pub eth_address: String,
    pub did: String,
    pub fragment: String,
    pub vcredential: Option<String>,
    pub credential_id: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityRequest {
    pub eth_address: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialRequest {
    pub credential_jwt: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub alternate_name: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    #[serde(rename = "type")]
    pub service_type: String,
    pub service_endpoint: Url
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub credential: Credential,
    pub services: Option<Vec<Service>>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignDataRequest {
    pub payload: String, // TODO: bytes
    pub nonce: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationRequest {
    pub challenge: String,
    pub eth_signature: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetUploadRequest {
    pub eth_address: String,
    pub alias: String,
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetUpdateRequest {
    pub nft_address: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryEthAddress {
    pub eth_address: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryAssetAlias {
    pub alias: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChallengeResponse {
    pub nonce: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProofOfPurchaseRequest {
    pub nonce: String,
    pub eth_address: String,
    pub eth_signature: String,
    pub did: String,
}