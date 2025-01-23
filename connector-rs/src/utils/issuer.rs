// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::str::FromStr;
use alloy::hex::ToHexExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use crate::{dtos::CredentialData, errors::ConnectorError, models::{self, identity::{CredentialIssuedResponse, Identity}}};

use super::iota::IotaState;

#[derive(Debug, Clone)]
pub struct Issuer{
    base_url: Url,
    client: reqwest::Client
}

#[derive(Serialize, Deserialize)]
struct Challenge{
    nonce: String
}


impl Issuer{
    /// Create a Issuer handler
    /// ### Fields
    /// - url: base address of a supported Mediterraneus Issuer
    pub fn init(url: impl AsRef<str>) -> anyhow::Result<Self>{
        let uri = Url::from_str(url.as_ref())?;
        let client = reqwest::Client::builder()
            .build()?;
        Ok(Self{base_url: uri, client:client})
    }

    /// Attempt to register an identity to the issuer
    pub async fn register(&self, identity: &models::identity::Identity, iota_state: &IotaState, credential_data: CredentialData) -> Result<Identity, ConnectorError>{
        let mut challenge_url = self.base_url.clone();
        let client = &self.client;

        challenge_url.set_path("/api/challenges");
        let did = format!("did={}", identity.did);
        challenge_url.set_query(Some(did.as_str()));

        let challenge: Challenge = client.get(challenge_url)
            .send()
            .await?
            .json::<Challenge>()
            .await?;

        let ssi_signature = iota_state.sign_data(identity.clone(), challenge.nonce.as_bytes().to_vec(), &Some(challenge.nonce.clone())).await?;
        let evm_signature = iota_state.sign_evm_data(challenge.nonce.as_bytes()).await?;

        let evm_signature = evm_signature.encode_hex_with_prefix();

        let subject = json!(credential_data);

        let credential_req_body = json!({
            "did": identity.did,
            "nonce": &challenge.nonce,
            "identitySignature": ssi_signature.as_str(),
            "walletSignature": evm_signature,
            "credentialSubject": subject
        });

        let mut credential_url = self.base_url.clone();
        credential_url.set_path("/api/credentials");

        let credential: CredentialIssuedResponse = client.post(credential_url)
            .json(&credential_req_body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mut identity_with_cred = identity.clone();
        identity_with_cred.vcredential = Some(credential.credential_jwt.as_str().to_owned());
        Ok(identity_with_cred)
    } 

    /// Request to issuer to revoke credential
    pub async fn revoke_vc(&self, credential_id: &str) -> Result<(), ConnectorError>{
        let mut issuer_url = self.base_url.clone();
        let client = &self.client;

        //build path for revocation endopoint
        issuer_url.set_path(format!("/api/credentials/{}", credential_id).as_str());

        client.delete(issuer_url)
        .send()
        .await?
        .error_for_status_ref()?;
    
        Ok(())
    }
}