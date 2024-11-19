// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::str::FromStr;
use deadpool_postgres::Pool;
use ethers::utils::hex::ToHexExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use crate::{models::{self, identity::{CredentialIssuedResponse, Identity}}, repository::identity_operations::IdentityExt};

use super::iota::IotaState;

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
    pub async fn try_register(&self, identity: &models::identity::Identity, iota_state: &IotaState, db_pool: &Pool) -> anyhow::Result<Identity>{
        let pg_client = db_pool.get().await.map_err(crate::errors::ConnectorError::PoolError)?;

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

        let subject = json!({
            "id": identity.did,
            "name": "DLT",
            "surname": "Connector"
        });

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

        
        let identity = pg_client.set_credential(&identity.eth_address, &Some(credential.credential_jwt.as_str().to_owned())).await?;
        Ok(identity)
    } 
}