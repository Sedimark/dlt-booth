// Copyright 2020-2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;
use std::str::FromStr;

use alloy::signers::Signer;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;

use crypto::keys::bip44::Bip44;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::core::Object;
use identity_iota::credential::Jws;
use identity_iota::credential::Jwt;
use identity_iota::credential::JwtPresentationOptions;
use identity_iota::credential::Presentation;
use identity_iota::credential::PresentationBuilder;
use identity_iota::did::CoreDID;
use identity_iota::did::DIDUrl;
use identity_iota::did::RelativeDIDUrl;
use identity_iota::did::DID;
use identity_iota::document::verifiable::JwsVerificationOptions;

use identity_iota::document::ServiceBuilder;
use identity_iota::document::ServiceEndpoint;
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
use identity_iota::verification::MethodBuilder;
use identity_iota::verification::MethodData;
use identity_iota::verification::MethodScope;

use identity_iota::verification::MethodType;
use identity_iota::verification::jws::JwsAlgorithm;
use identity_stronghold::StrongholdStorage;
use iota_sdk::client::api::GetAddressesOptions;
use iota_sdk::client::constants::SHIMMER_COIN_TYPE;
use iota_sdk::client::secret::GenerateAddressOptions;
use iota_sdk::client::secret::SecretManager;
use iota_sdk::client::Password;
use iota_sdk::client::node_api::indexer::query_parameters::QueryParameter;
use iota_sdk::client::secret::stronghold::StrongholdSecretManager;
use iota_sdk::client::Client;
use iota_sdk::types::block::address::Bech32Address;
use iota_sdk::wallet::WalletBuilder;
use iota_sdk::Wallet;
use serde_json::Value;
use serde_json::json;

use crate::errors::ConnectorError;
use crate::models::identity::Identity;

use super::alloy_signer::IotaSigner;
use super::configs::DLTConfig;
use super::configs::EvmAddressConfig;
use super::configs::KeyStorageConfig;
use super::configs::WalletStorageConfig;


pub type MemStorage = Storage<StrongholdStorage, StrongholdStorage>;

pub struct IotaState {
  pub wallet: Wallet,
  pub stronghold_storage: StrongholdStorage,
  pub key_storage: MemStorage,
  pub address: Bech32Address,
  pub evm_address_chain: Bip44,
  pub dlt_config: DLTConfig
}

impl std::fmt::Debug for IotaState{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IotaState")
        .field("wallet", &self.wallet)
        .field("stronghold_storage", &self.stronghold_storage)
        .field("key_storage", &self.key_storage.key_storage())
        .field("key_id_storage", &self.key_storage.key_id_storage())
        .field("address", &self.address)
        .field("dlt_config", &self.dlt_config)
        .finish()
    }
}

impl IotaState {
  // create_or_recover_key_storage
  pub async fn init(key_storage_config: KeyStorageConfig, 
    wallet_config: WalletStorageConfig, 
    dlt_config: DLTConfig,
    evm_address_config: EvmAddressConfig) -> Result<Self> {
    log::info!("Creating or recovering storage...");

    let node_url = &dlt_config.node_url;
    
    
    // Setup Stronghold secret_manager
    let stronghold = StrongholdSecretManager::builder()
    .password(Password::from(key_storage_config.password.value()))
    .build(key_storage_config.file_path)?;

    // Only used the first time, if there is a mnemonic stored within Stronghold it won't be used
    let mnemonic = Client::generate_mnemonic()?;

    match stronghold.store_mnemonic(mnemonic).await {
      Ok(())=> log::info!("Stronghold mnemonic stored (Key storage)"),
      Err(iota_sdk::client::stronghold::Error::MnemonicAlreadyStored) => log::debug!("Stronghold mnemonic already stored (Key storage)"),
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
    .with_node(&node_url)?;

    // Generate a Wallet
    let wallet_stronghold = StrongholdSecretManager::builder()
    .password(wallet_config.password.value())
    .build(wallet_config.file_path)?;

    // Only used the first time, if there is a mnemonic stored within Stronghold it won't be used
    let wallet_mnemonic = Client::generate_mnemonic()?;

    match wallet_stronghold.store_mnemonic(wallet_mnemonic).await{
      Ok(()) => log::info!("Stronghold mnemonic stored (Wallet)"),
      Err(iota_sdk::client::stronghold::Error::MnemonicAlreadyStored) => log::debug!("Stronghold mnemonic already stored (Wallet)"),
      Err(error) => panic!("Error: {:?}", error)      
    }

    let wallet = WalletBuilder::new()
      .with_client_options(client)
      .with_secret_manager(SecretManager::Stronghold(wallet_stronghold))
      .with_coin_type(SHIMMER_COIN_TYPE) // TODO: is it possible to define the coin type from the node or chain id
      .finish()
      .await?;

    // Generates or recover an address from the given [`SecretManager`] and adds funds from the faucet.
    let identity_account = wallet.get_or_create_account("identity").await?;
    let bech_address;
    if let Some(address) = identity_account.addresses().await?.first(){
      bech_address = address.clone().into_bech32();
    }
    else {
      bech_address = identity_account
        .generate_ed25519_addresses(1, GenerateAddressOptions::default()).await?.first()
        .ok_or(anyhow!("Wallet cannot generate addresses"))
        .and_then(|address| {Ok(address.clone().into_bech32())})?;
    }

    let iota_state = IotaState{ 
      wallet, 
      stronghold_storage, 
      key_storage: storage, 
      address: bech_address,
      dlt_config,
      evm_address_chain: evm_address_config.into() };

    iota_state.ensure_address_has_funds().await?;
    Ok(iota_state)
  }

  /// Creates a DID Document and publishes it in a new Alias Output.
  ///
  /// Its functionality is equivalent to the "create DID" example
  /// and exists for convenient calling from the other examples.
  pub async fn create_did(
    &self,
    eth_address: Option<impl ToString>,
    services: Option<Vec<crate::dtos::Service>>
  ) -> Result<(IotaDocument, String), ConnectorError> {
        
    let (document, fragment): (IotaDocument, String) = Self::create_did_document( &self, eth_address, services).await?;
    let client = self.wallet.client();
    let alias_output: AliasOutput = client.new_did_output(self.address.into_inner(), document, None).await?;

    let secret_manager = self.wallet.get_secret_manager().clone().try_read_owned()?;
    let document: IotaDocument = client.publish_did_output(
      &secret_manager,
      alias_output
    ).await?;

    Ok((document, fragment))
  }

  /// Creates an example DID document with the given `network_name`.
  ///
  /// Its functionality is equivalent to the "create DID" example
  /// and exists for convenient calling from the other examples.
  async fn create_did_document(
    &self,
    eth_address: Option<impl ToString>,
    services: Option<Vec<crate::dtos::Service>>
  ) -> Result<(IotaDocument, String), ConnectorError> {
  let network_name: NetworkName = self.wallet.client().network_name().await?;
  let mut document: IotaDocument = IotaDocument::new(&network_name);

  let fragment: String = document
  .generate_method(
    &self.key_storage,
    JwkMemStore::ED25519_KEY_TYPE,
    JwsAlgorithm::EdDSA,
    None,
    MethodScope::VerificationMethod,
  )
  .await?;

  let did = CoreDID::from(document.id().clone());
  // Add services to DID Document
  if let Some(service_list) = services {
    let service_builder = service_list.iter().filter_map(|service| {
      let mut service_id = RelativeDIDUrl::new();
      service_id.set_fragment(Some(service.id.as_str())).ok()?;
      let service_id = DIDUrl::new(did.clone(), Some(service_id));

      ServiceBuilder::new(Object::new())
        .id(service_id)
        .type_(service.service_type.clone())
        .service_endpoint(ServiceEndpoint::One(service.service_endpoint.clone().into()))
        .build().ok()
    });

    for service in service_builder{
      document.insert_service(service)?;
    }

  }
  
  eth_address.map(|eth_address| -> Result<(), ConnectorError> {
    let mut properties = BTreeMap::new();
    properties.insert("blockchainAccountId".to_string(), json!(format!("eip155:1:{}", eth_address.to_string())));

    let id = document.id().to_url().join("#ethAddress")?;

    log::info!("id: {}", id.to_string());
    // Add eth addr as verification method: https://www.w3.org/TR/did-spec-registries/#blockchainaccountid 
    let method = MethodBuilder::new(properties)
      .id( id )
      .type_(MethodType::from_str("EcdsaSecp256k1RecoverySignature2020")?)
      .controller(document.core_document().id().to_owned())
      .data(MethodData::PublicKeyMultibase("".into()))
      .build().unwrap();
    document.insert_method(method, MethodScope::VerificationMethod)?;
    Ok(())
  });
  
  Ok((document, fragment))
  }

  pub async fn resolve_did(
    &self,
    did: &str
  ) -> Result<IotaDocument, ConnectorError> {
    log::info!("Resolving did...");
    log::info!("{}/identity-resolver/{}", self.dlt_config.explorer_url, did);
    match self.wallet.client().resolve_did(&IotaDID::try_from(did)?).await {
        Ok(iota_document) => Ok(iota_document),
        Err(err) => {
          log::info!("Error {}", err);
          Err(ConnectorError::ResolveError(err))
        },
    }
  }

  /// Removes did output from the DLT.
  /// 
  /// The did is dropped and the resolve operation will fail
  pub async fn delete_did(&self, did: &IotaDID) -> Result<(), ConnectorError> {
    let client = self.wallet.client();
    let secret_manager = self.wallet.get_secret_manager().clone().try_read_owned()?;
    
    client.delete_did_output(&secret_manager, *self.address, did).await?;
    
    Ok(())
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
    iota_sdk::client::request_funds_from_faucet(&self.dlt_config.faucet_api_endpoint, &self.address).await?;

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
    let client = self.wallet.client();
    let output_ids = client
      .basic_output_ids(vec![
        QueryParameter::Address(self.address.to_owned()),
        QueryParameter::HasExpiration(false),
        QueryParameter::HasTimelock(false),
        QueryParameter::HasStorageDepositReturn(false),
      ])
      .await?;

    let outputs = client.get_outputs(&output_ids).await?;

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
    nonce: &Option<String>
  ) -> Result<Jws, ConnectorError> {
    log::info!("Resolving did...");
    let document = self.resolve_did(identity.did.as_str()).await?;

    log::info!("create_jws");

    let (jws_signature_options, jws_verification_options) = match nonce {
        Some(nonce) => (JwsSignatureOptions::default().nonce(nonce), JwsVerificationOptions::default().nonce(nonce)),
        None =>  (JwsSignatureOptions::default(), JwsVerificationOptions::default()),
    };

    // Compute signature
    let jws = document.create_jws(&self.key_storage, &identity.fragment, &payload, &jws_signature_options).await?;
    // Verify signature
    let _decoded_jws = document.verify_jws(
        &jws,
        None,
        &EdDSAJwsVerifier::default(),
        &jws_verification_options,
    )?; 

    Ok(jws)
  }

  pub async fn gen_presentation(
    &self,
    identity: Identity,
    challenge: String,
    wallet_signature_claim: Option<BTreeMap<String, Value>>
  ) -> Result<Jwt, ConnectorError> {
    log::info!("Resolving did...");
    let document = self.resolve_did(identity.did.as_str()).await?;

    let credential_jwt = Jwt::new(identity.vcredential.ok_or(ConnectorError::CredentialMissing)?);

    log::info!("gen_presentation");
    // Create an unsigned Presentation from the previously issued Verifiable Credential.

    let presentation: Presentation<Jwt> =
    PresentationBuilder::new(document.id().to_url().into(), Default::default())
      .credential(credential_jwt)
      .build()?;
  
   let jwt_presentation_options = match wallet_signature_claim {
      Some(wallet_signature_claim) => {
        let mut jwt_presentation_options = JwtPresentationOptions::default();
        jwt_presentation_options.custom_claims = Some(wallet_signature_claim); 
        jwt_presentation_options
      },
      None => JwtPresentationOptions::default(),
    };

    // Create a JWT verifiable presentation using the holder's verification method
    // and include the requested challenge and expiry timestamp.
    let presentation_jwt: Jwt = document
      .create_presentation_jwt(
        &presentation,
        &self.key_storage,
        &identity.fragment,
        &JwsSignatureOptions::default().nonce(challenge.to_owned()),
        &jwt_presentation_options // .expiration_date(expires), // TODO: add expiration handling
      )
      .await?; 
    log::info!("{:?}",presentation_jwt);
    Ok(presentation_jwt)
  }
  
  /// Return a signer containing an instance of SecretManager
  pub async fn get_evm_signer<'a>(&self, secret_manager : &'a SecretManager) -> Result<IotaSigner<'a>, ConnectorError>{
    IotaSigner::new(secret_manager, Some(self.dlt_config.chain_id), self.evm_address_chain).await
  }

  pub async fn get_evm_address(&self) -> Result<String, ConnectorError>{
    let secret_manager = self.wallet.get_secret_manager().try_read()?;
    let address_index = self.evm_address_chain.address_index;
    let options = GetAddressesOptions::default()
      .with_coin_type(self.evm_address_chain.coin_type)
      .with_account_index(self.evm_address_chain.account)
      .with_range(address_index..address_index+1);
    secret_manager
      .generate_evm_addresses(options)
      .await?
      .first()
      .ok_or(ConnectorError::WalletError("Cannot access to wallet".to_owned()))
      .map(|res| res.to_owned())
  }

  pub async fn sign_evm_data(&self, payload: impl AsRef<[u8]>) -> Result<Vec<u8>, ConnectorError>{
    let secret_manager = self.wallet.get_secret_manager().try_read()?;
    let signer = self.get_evm_signer(&secret_manager).await?;
    Ok(signer.sign_message(payload.as_ref()).await?.as_bytes().to_vec())
  }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use url::Url;

    use crate::utils::configs::{ConfigSecret, DLTConfig, EvmAddressConfig, KeyStorageConfig, WalletStorageConfig};

    use super::IotaState;

  #[tokio::test]
  async fn evm_address_test(){
    let key_storage = KeyStorageConfig{ 
      file_path:"test_ks.stronghold".to_owned(), 
      password: ConfigSecret::from_str("some_hopefully_secure_password").unwrap()}; 
    let wallet_storage = WalletStorageConfig{ 
      file_path:"test_w.stronghold".to_owned(), 
      password: ConfigSecret::from_str("some_hopefully_secure_password").unwrap()};
    
    let dlt_config = DLTConfig{ rpc_provider: Url::from_str("https://json-rpc.evm.testnet.shimmer.network").unwrap(),
      chain_id: 1073,
      node_url: "https://api.testnet.shimmer.network".to_owned(),
      faucet_api_endpoint: "https://faucet.testnet.shimmer.network/api/enqueue".to_owned(),
      explorer_url: "".to_owned(),
      issuer_url: "http://localhost:3213".to_owned(),
      factory_sc_address: "".to_string(),
    };

    let evm_config = EvmAddressConfig::default().with_coin_type(60).with_address_index(0);
    let iota = IotaState::init(key_storage, wallet_storage, dlt_config, evm_config).await.unwrap();
    let address = iota.get_evm_address().await.unwrap();

    assert_eq!(address, "0xbd73f73c8c096e1a520a3380b1df3ca3fd5ac3c3")

  }
}