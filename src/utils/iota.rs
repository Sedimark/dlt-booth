// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// SPDX-FileCopyrightText: 2023 Fondazione LINKS
// SPDX-License-Identifier: APACHE-2.0

use anyhow::Result;

use crypto::keys::bip39::Mnemonic;
use identity_iota::iota::block::output::AliasOutput;
use identity_iota::iota::IotaClientExt;
use identity_iota::iota::IotaDocument;
use identity_iota::iota::IotaIdentityClientExt;
use identity_iota::iota::NetworkName;
use identity_iota::storage::JwkDocumentExt;
use identity_iota::storage::JwkMemStore;
use identity_iota::storage::Storage;
use identity_iota::verification::MethodScope;

use identity_iota::verification::jws::JwsAlgorithm;
use identity_stronghold::StrongholdStorage;
use iota_sdk::client::Password;
use iota_sdk::client::secret::stronghold::StrongholdSecretManager;
use iota_sdk::client::Client;
use iota_sdk::types::block::address::Address;

use crate::errors::ConnectorError;

pub type MemStorage = Storage<StrongholdStorage, StrongholdStorage>;

pub struct IotaState {
  client: Client,
  stronghold_storage: StrongholdStorage,
  storage: MemStorage,
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

    Ok(IotaState{ client, stronghold_storage, storage})
  }

  /// Creates a DID Document and publishes it in a new Alias Output.
  ///
  /// Its functionality is equivalent to the "create DID" example
  /// and exists for convenient calling from the other examples.
  pub async fn create_did(
    &self,
    address: Address
  ) -> anyhow::Result<(Address, IotaDocument, String)> {
    // TODO: check and eventually remove this
    // let address: Address = get_address_with_funds(client, secret_manager, FAUCET_ENDPOINT)
    //   .await
    //   .context("failed to get address with funds")?;
    
    let (document, fragment): (IotaDocument, String) = Self::create_did_document( &self).await?;

    let alias_output: AliasOutput = self.client.new_did_output(address, document, None).await?;

    let document: IotaDocument = self.client.publish_did_output(
      self.stronghold_storage.as_secret_manager(),
      alias_output
    ).await?;

    Ok((address, document, fragment))
  }

  /// Creates an example DID document with the given `network_name`.
  ///
  /// Its functionality is equivalent to the "create DID" example
  /// and exists for convenient calling from the other examples.
  async fn create_did_document(
    &self
  ) -> anyhow::Result<(IotaDocument, String)> {
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

}
