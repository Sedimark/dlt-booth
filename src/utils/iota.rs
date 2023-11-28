// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// SPDX-FileCopyrightText: 2023 Fondazione LINKS
// SPDX-License-Identifier: APACHE-2.0

use anyhow::Context;
use anyhow::Result;

use crypto::keys::bip39::Mnemonic;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::credential::Jws;
use identity_iota::document::verifiable::JwsVerificationOptions;
use identity_iota::iota::IotaDID;
use identity_iota::iota::block::output::AliasOutput;
use identity_iota::iota::IotaClientExt;
use identity_iota::iota::IotaDocument;
use identity_iota::iota::IotaIdentityClientExt;
use identity_iota::iota::NetworkName;
use identity_iota::storage::JwkDocumentExt;
use identity_iota::storage::JwkMemStore;
use identity_iota::storage::JwsSignatureOptions;
use identity_iota::storage::Storage;
use identity_iota::verification::MethodScope;

use identity_iota::verification::jws::JwsAlgorithm;
use identity_stronghold::StrongholdStorage;
use iota_sdk::client::Password;
use iota_sdk::client::api::GetAddressesOptions;
use iota_sdk::client::node_api::indexer::query_parameters::QueryParameter;
use iota_sdk::client::secret::stronghold::StrongholdSecretManager;
use iota_sdk::client::Client;
use iota_sdk::types::block::address::Bech32Address;
use iota_sdk::types::block::address::Hrp;

use crate::errors::ConnectorError;
use crate::models::identity::Identity;

pub type MemStorage = Storage<StrongholdStorage, StrongholdStorage>;

pub struct IotaState {
  pub client: Client,
  pub stronghold_storage: StrongholdStorage,
  pub storage: MemStorage,
  pub address: Bech32Address,
  pub faucet_url: String
}

impl IotaState {
  // create_or_recover_key_storage
  pub async fn init() -> Result<Self> {
    log::info!("Creating or recovering storage...");

    let stronghold_pass = std::env::var("KEY_STORAGE_STRONGHOLD_PASSWORD")
    .expect("$KEY_STORAGE_STRONGHOLD_PASSWORD must be set.");

    let stronghold_path = std::env::var("KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH")
    .expect("$KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH must be set.");

    let mnemonic_string = std::env::var("KEY_STORAGE_MNEMONIC")
    .expect("$KEY_STORAGE_MNEMONIC must be set.");

    let node_url = std::env::var("NODE_URL").expect("$NODE_URL must be set.");
    let faucet_url = std::env::var("FAUCET_URL").expect("$FAUCET_URL must be set.");
    
    
    // Setup Stronghold secret_manager
    let stronghold = StrongholdSecretManager::builder()
    .password(Password::from(stronghold_pass))
    .build(stronghold_path)?;

    // Only required the first time, can also be generated with `manager.generate_mnemonic()?`
    let mnemonic = Mnemonic::from(mnemonic_string);

    match stronghold.store_mnemonic(mnemonic).await {
      Ok(()) => log::info!("Stronghold mnemonic stored"),
      Err(iota_sdk::client::stronghold::Error::MnemonicAlreadyStored) => log::info!("Stronghold mnemonic already stored"),
      Err(error) => panic!("Error: {:?}", error)
    }

    // Create a `StrongholdStorage`.
    // `StrongholdStorage` creates internally a `SecretManager` that can be
    // referenced to avoid creating multiple instances around the same stronghold snapshot.
    let stronghold_storage = StrongholdStorage::new(stronghold);

    // Create storage for key-ids and JWKs.
    //
    // In this example, the same stronghold file that is used to store
    // key-ids as well as the JWKs.
    let storage = Storage::new(stronghold_storage.clone(), stronghold_storage.clone());

    let client = Client::builder()
    .with_node(&node_url)?
    .finish()
    .await
    .map_err(|e| ConnectorError::from(e))?;

    // let address: Address = Self::get_address_with_funds(&client, stronghold_storage.as_secret_manager(), faucet_url.as_str())
    // .await
    // .context("failed to get address with funds")?;

    // TODO: try recover this address, or gen and store 
    // Generates or recover an address from the given [`SecretManager`] and adds funds from the faucet.
    let bech32_hrp: Hrp = client.get_bech32_hrp().await?;
    let address: Bech32Address = stronghold_storage.as_secret_manager()
    .generate_ed25519_addresses(
      GetAddressesOptions::default()
        .with_range(0..1)
        .with_bech32_hrp(bech32_hrp),
    )
    .await?[0];

    let iota_state = IotaState{ client, stronghold_storage, storage, address, faucet_url };

    iota_state.ensure_address_has_funds().await?;

    Ok(iota_state)
  }

  /// Creates a DID Document and publishes it in a new Alias Output.
  ///
  /// Its functionality is equivalent to the "create DID" example
  /// and exists for convenient calling from the other examples.
  pub async fn create_did(
    &self,
  ) -> Result<(IotaDocument, String), ConnectorError> {
        
    let (document, fragment): (IotaDocument, String) = Self::create_did_document( &self).await?;

    let alias_output: AliasOutput = self.client.new_did_output(self.address.into_inner(), document, None).await?;

    let document: IotaDocument = self.client.publish_did_output(
      self.stronghold_storage.as_secret_manager(),
      alias_output
    ).await?;

    Ok((document, fragment))
  }

  /// Creates an example DID document with the given `network_name`.
  ///
  /// Its functionality is equivalent to the "create DID" example
  /// and exists for convenient calling from the other examples.
  async fn create_did_document(
    &self
  ) -> Result<(IotaDocument, String), ConnectorError> {
    let network_name: NetworkName = self.client.network_name().await?;
    let mut document: IotaDocument = IotaDocument::new(&network_name);

    let fragment: String = document
    .generate_method(
      &self.storage,
      JwkMemStore::ED25519_KEY_TYPE,
      JwsAlgorithm::EdDSA,
      None,
      MethodScope::VerificationMethod,
    )
    .await?;

    Ok((document, fragment))
  }

  pub async fn resolve_did(
    &self,
    did: &str
  ) -> Result<IotaDocument, ConnectorError> {
    log::info!("Resolving did...");
    log::info!("{}/identity-resolver/{}", std::env::var("EXPLORER_URL").unwrap(), did);
    match self.client.resolve_did(&IotaDID::try_from(did)?).await {
        Ok(iota_document) => Ok(iota_document),
        Err(err) => {
          log::info!("Error {}", err);
          Err(ConnectorError::ResolveError(err))
        },
    }
  }

  /// Requests funds from the faucet for the given `address` if it has not enough funds.
  pub async fn ensure_address_has_funds(&self) -> anyhow::Result<()> {

    let balance = self.get_address_balance()
    .await
    .context("failed to get address balance")?;
  
    if balance == 0 {
      log::info!("Funding address");
      self.request_faucet_funds().await?;
  
    } else {
      log::info!("Address has already enough funds: {}.", balance);
    }
    Ok(())
  }
  

  /// Requests funds from the faucet for the given `address`.
  async fn request_faucet_funds(&self) -> anyhow::Result<()> {
    iota_sdk::client::request_funds_from_faucet(&self.faucet_url, &self.address).await?;

    tokio::time::timeout(std::time::Duration::from_secs(45), async {
      loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let balance = self.get_address_balance()
          .await
          .context("failed to get address balance")?;
        if balance > 0 {
          break;
        }
      }
      Ok::<(), anyhow::Error>(())
    })
    .await
    .context("maximum timeout exceeded")??;

    Ok(())
  }

  /// Returns the balance of the given Bech32-encoded `address`.
  async fn get_address_balance(&self) -> anyhow::Result<u64> {
    let output_ids = self.client
      .basic_output_ids(vec![
        QueryParameter::Address(self.address.to_owned()),
        QueryParameter::HasExpiration(false),
        QueryParameter::HasTimelock(false),
        QueryParameter::HasStorageDepositReturn(false),
      ])
      .await?;

    let outputs = self.client.get_outputs(&output_ids).await?;

    let mut total_amount = 0;
    for output_response in outputs {
      total_amount += output_response.output().amount();
    }

    Ok(total_amount)
  }

  pub async fn sign_data(
    &self,
    identity: Identity,
    payload: Vec<u8>,
  ) -> Result<Jws, ConnectorError> {
    log::info!("Resolving did...");
    let document = self.resolve_did(identity.did.as_str()).await?;

    log::info!("create_jws");
    // Compute signature
    let jws = document.create_jws(&self.storage, &identity.fragment, &payload, &JwsSignatureOptions::default()).await?;
    // Verify signature
    let _decoded_jws = document.verify_jws(
        &jws,
        None,
        &EdDSAJwsVerifier::default(),
        &JwsVerificationOptions::default(),
    )?; 

    Ok(jws)
  }
  
}
