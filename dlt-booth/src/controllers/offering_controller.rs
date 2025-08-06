// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later
use std::{error::Error, str::FromStr, time::Duration};

use actix_web::{get, post, web, HttpResponse, Responder};
use alloy::{network::Ethereum, primitives::{utils::parse_ether, Address, U256}, providers::{DynProvider, Provider, ProviderBuilder}};
use deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::json;
use crate::{contracts::{Factory::{self, PublishData}, ServiceBase}, errors::ConnectorError, repository::evm_data_operations::EvmAddressesExt, utils::{iota::IotaState, issuer::Issuer, stronghold_local_wallet::StrongholdWallet}};

#[derive(Deserialize)]
struct Addresses{
    factory: String,
}

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

#[post("/delegated/offerings")]
async fn publish_offering(
    db_pool: web::Data<Pool>,
    iota_state: web::Data<IotaState>,
    offering: web::Json<OfferingData>
) -> Result<HttpResponse, ConnectorError>{
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let factory_address = pg_client.get_address("factory").await?;
    
    let secret_manager = iota_state.wallet.get_secret_manager().try_read()?;
    let signer = iota_state.get_evm_signer(&secret_manager).await?;
    let signer = StrongholdWallet::new(signer);
    let provider = ProviderBuilder::new()
    .network::<Ethereum>()
    .wallet(signer)
    .connect_http(iota_state.dlt_config.rpc_provider.clone());
    provider.client().set_poll_interval(Duration::from_millis(50));

    let factory = Factory::new(factory_address, provider);

    // compute nft address
    let call_builder = factory.tokenizeService(offering.into_inner().try_into()?)
      .gas_price(10_000_000_000);

    let nft_address = call_builder
        .call()
        .await
        .map_err(|e| ConnectorError::OtherError(e.to_string()))?;
    let nft_address = nft_address.to_string();

    // execute transaction and wait for confirmation
    call_builder.send().await
      .map_err(|e| ConnectorError::OtherError(e.to_string()))?
      .watch()
      .await
      .map_err(|e| ConnectorError::OtherError(e.to_string()))?;
    Ok(HttpResponse::Created().json(json!({"nftAddress": nft_address})))
}

#[get("/offerings")]
async fn get_offerings(
  db_pool: web::Data<Pool>,
  sc_provider: web::Data<DynProvider>,
  issuer: web::Data<Issuer>
) -> Result<impl Responder, ConnectorError>{

  // Retrieve factory address
  let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
  
  log::debug!("Reading Factory SC address...");
  let factory_address = match  pg_client.get_address("factory").await
  {
    Ok(address) => address,
    Err(ConnectorError::TokioPostgresError(e)) if format!("{:?}",e).contains("RowCount") => {
      log::debug!("Address not found in DB. Retrieve SC");
      let address = issuer.get_addresses::<Addresses>().await?;
      pg_client.insert_address("factory", &address.factory).await?;
      Address::from_str(&address.factory)?
    }
    Err(e) => {
        log::error!("{:?}",e.source().unwrap().to_string());
        return Err(e)
    }
  };

  let factory = Factory::new(factory_address, sc_provider.into_inner());
  let result = factory
    .getAllNFTCreatedAddress()
    .call()
    .await
    .map_err(|e| ConnectorError::OtherError(e.to_string()))?
    .iter().map(|addr| addr.to_string())
    .collect::<Vec<String>>();
  Ok(HttpResponse::Ok().json(json!({"addresses": result})))
}

#[get("/offerings/{nft_address}")]
async fn get_offering(
  path: web::Path<String>,
  sc_provider: web::Data<DynProvider>
) -> Result<impl Responder, ConnectorError>{

  let nft_address = Address::from_str(&path)?;

  let servicebase = ServiceBase::new(nft_address, sc_provider.into_inner());
  let owner = servicebase.getServiceOwner()
    .call().await
    .map_err(|e| ConnectorError::OtherError(e.to_string()))?
    .to_string();
  let nft_name = servicebase.name()
    .call().await
    .map_err(|e| ConnectorError::OtherError(e.to_string()))?;
  let description_uri = servicebase.tokenURI(U256::from(1))
    .call().await
    .map_err(|e| ConnectorError::OtherError(e.to_string()))?;

  let description_hash = servicebase.getDescriptionHash()
    .call().await
    .map_err(|e| ConnectorError::OtherError(e.to_string()))?;
  
  Ok(HttpResponse::Ok().json(json!({
    "owner": owner,
    "name": nft_name,
    "descriptionUri": description_uri,
    "descriptionHash": description_hash
  })))
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(publish_offering)
    .service(get_offerings)
    .service(get_offering);
}