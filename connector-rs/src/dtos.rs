// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use actix_multipart::form::{
    tempfile::TempFile,
    MultipartForm, text::Text,
};

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
pub struct CredentialData {
    pub name: String,
    pub surname: String
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

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(rename = "files")]
    pub files: Vec<TempFile>,
    #[multipart(rename = "ethAddress")]
    pub eth_address: Text<String>,
    pub alias: Text<String>

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