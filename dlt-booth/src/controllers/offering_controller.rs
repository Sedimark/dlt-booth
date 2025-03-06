use std::str::FromStr;

use actix_web::{get, post, web, HttpResponse, Responder};
use alloy::{network::{AnyNetwork, NetworkWallet}, primitives::{utils::parse_ether, Address, U256}, providers::{ProviderBuilder, WalletProvider}};
use serde::Deserialize;
use serde_json::json;
use crate::{contracts::{Factory::{self, PublishData}, ScProvider, ServiceBase}, errors::ConnectorError, utils::{iota::IotaState, stronghold_local_wallet::StrongholdWallet}};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OfferingData{
    name: String,
    symbol: String,
    description_uri: String, 
    description_hash: String,
    dt_name: String,
    dt_symbol: String,
    max_supply: u64
}

impl TryFrom<OfferingData> for PublishData{
  type Error = ConnectorError;

  fn try_from(value: OfferingData) -> Result<Self, Self::Error> {
    let max_supply = parse_ether(&value.max_supply.to_string())
      .map_err(|e| ConnectorError::OtherError(e.to_string()))?;

    Ok(PublishData{
      name: value.name,
      symbol: value.symbol,
      descriptionUri: value.description_uri,
      descriptionHash: value.description_hash,
      dt_name: value.dt_name,
      dt_symbol: value.dt_symbol,
      maxSupply_: max_supply
    })
  }
}

#[post("/offerings")]
async fn publish_offering(
    iota_state: web::Data<IotaState>,
    offering: web::Json<OfferingData>
) -> Result<HttpResponse, ConnectorError>{
    let factory_address = Address::from_str(&iota_state.dlt_config.factory_sc_address)?;
    let secret_manager = iota_state.wallet.get_secret_manager().try_read()?;
    let signer = iota_state.get_evm_signer(&secret_manager).await?;
    let signer = StrongholdWallet::new(signer);
    let provider = ProviderBuilder::new()
    .with_recommended_fillers()
    .wallet(signer)
    .on_http(iota_state.dlt_config.rpc_provider.clone());

    let factory = Factory::new(factory_address, provider);
    log::error!("Default {:?}", factory.provider().default_signer_address());

    let nft_address = factory.tokenizeService(offering.into_inner().try_into()?)
        .gas_price(10_000_000_000)
        .call()
        .await
        .map_err(|e| ConnectorError::OtherError(e.to_string()))?;
    let nft_address = nft_address.erc721token.to_string();
    Ok(HttpResponse::Created().json(json!({"nft_address": nft_address})))
}

#[get("/offerings")]
async fn get_offerings(
  iota_state: web::Data<IotaState>,
  sc_provider: web::Data<ScProvider>
) -> Result<impl Responder, ConnectorError>{

  let factory_address = Address::from_str(&iota_state.dlt_config.factory_sc_address)?;
  let factory = Factory::new(factory_address, sc_provider.into_inner());
  let result = factory
    .getAllNFTCreatedAddress()
    .call()
    .await
    .map_err(|e| ConnectorError::OtherError(e.to_string()))?
    ._0
    .iter().map(|addr| addr.to_string())
    .collect::<Vec<String>>();
  Ok(HttpResponse::Ok().json(json!({"addresses": result})))
}

#[get("/offerings/{nft_address}")]
async fn get_offering(
  path: web::Path<String>,
  sc_provider: web::Data<ScProvider>
) -> Result<impl Responder, ConnectorError>{

  let nft_address = Address::from_str(&path)?;

  let servicebase = ServiceBase::new(nft_address, sc_provider.into_inner());
  let owner = servicebase.getServiceOwner()
    .call().await
    .map_err(|e| ConnectorError::OtherError(e.to_string()))?
    .owner
    .to_string();
  let nft_name = servicebase.name()
    .call().await
    .map_err(|e| ConnectorError::OtherError(e.to_string()))?
    ._0;
  let description_uri = servicebase.tokenURI(U256::from(1))
    .call().await
    .map_err(|e| ConnectorError::OtherError(e.to_string()))?
    ._0;

  let description_hash = servicebase.getDescriptionHash()
    .call().await
    .map_err(|e| ConnectorError::OtherError(e.to_string()))?
    ._0;
  
  Ok(HttpResponse::Ok().json(json!({
    "owner": owner,
    "name": nft_name,
    "descriptionUri": description_uri,
    "description_hash": description_hash
  })))
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
      .service(publish_offering)
      .service(get_offerings)
      .service(get_offering);
}