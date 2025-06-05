// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use actix_web::{get, post, web::{self}, HttpResponse, Responder};
use alloy::{network::Ethereum, primitives::{utils::{parse_ether, Unit}, Address, FixedBytes}, providers::ProviderBuilder, sol_types::{SolEvent, SolValue}};
use crypto::hashes::keccak::{self};
use serde_json::json;
use std::str::FromStr;

use crate::{contracts::{AccessTokenBase, FixedRateExchange::{self, SuccessfulSwap}, ScProvider, ServiceBase}, errors::ConnectorError, utils::{iota::IotaState, stronghold_local_wallet::StrongholdWallet}};

/// Buy a new data token associated to the `nft_address` offering using dlt-booth identity
#[post("/delegated/dt/{nft_address}")]
async fn buy_dt(
  path: web::Path<String>,
  iota_state: web::Data<IotaState>,
  sc_provider: web::Data<ScProvider>
)
  -> Result<impl Responder, ConnectorError>
{

  // Retrieve the data token address associated to the offering
  let nft_address = Address::from_str(&path)?;
  let servicebase = ServiceBase::new(nft_address, sc_provider.clone().into_inner());

  let dt_address = servicebase
    .getATaddresses()
    .call()
    .await
    .ok()
    .and_then(|addresses| addresses.first().cloned())
    .ok_or(ConnectorError::OtherError("DT address not found".to_owned()))?;

  let access_token_base_contract = AccessTokenBase::new(dt_address, sc_provider.into_inner());
  let owner = access_token_base_contract.getDTowner()
    .call()
    .await
    .map_err(|e| ConnectorError::OtherError(e.to_string()))?;

  let fre_address = Address::from_str(&iota_state.dlt_config.fixed_rate_exchange_sc_address)?;
  let secret_manager = iota_state.wallet.get_secret_manager().try_read()?;
  let signer = iota_state.get_evm_signer(&secret_manager).await?;
  let signer = StrongholdWallet::new(signer);
  let provider = ProviderBuilder::new()
    .network::<Ethereum>()
    .wallet(signer)
    .connect_http(iota_state.dlt_config.rpc_provider.clone());
  let fixed_rate_exchange_contract = FixedRateExchange::new(fre_address, provider);

  // compute the exchange id
  let input = (dt_address, owner)
    .abi_encode();

  let mut exchange_id = [0 as u8; keccak::KECCAK256_LEN];
  keccak::keccak256(&input, &mut exchange_id);

  let smr_cost = fixed_rate_exchange_contract.getSMRcostFor1DT(alloy::primitives::FixedBytes(exchange_id))
    .call()
    .await
    .map_err(|_e| ConnectorError::OtherError("Cannot read contract details".to_owned()))?;

  let receipt = fixed_rate_exchange_contract.sellDT(FixedBytes(exchange_id), parse_ether("1").expect("Cannot parse a known value"))
    .gas_price(10_000_000_000)
    .value(smr_cost)
    .send()
    .await
    .map_err(|e| ConnectorError::OtherError(format!("Contract reverted: {}", e.to_string())))?
    // wait for confirmation
    .get_receipt()
    .await
    .map_err(|e| ConnectorError::OtherError(format!("Contract reverted: {}", e.to_string())))?;

  // check that the correct event has been emitted
  let swap_event_count = receipt
    .logs()
    .iter()
    .filter_map(|log| <SuccessfulSwap as SolEvent>::decode_log(&log.inner).ok())
    .inspect(|l| log::debug!("Successful swap event: {:?} --- {:?} --- {:?}", l.exchangeID, l.buyer, l.dtamountBougth))
    .count();

  // the transaction has been confirmed, return the balance of the account
  if swap_event_count == 1 {
    let booth_address = iota_state.get_evm_address()
      .await
      .and_then(|booth_addr| Ok(Address::from_str(&booth_addr)?))?;

    let balance: u32 = access_token_base_contract.balanceOf(booth_address)
      .call().await
      .map(|b| b / Unit::ETHER.wei_const())
      .map_err(|e| ConnectorError::OtherError(format!("Contract reverted: {}", e.to_string())))?
      .to();

    Ok(HttpResponse::Ok().json(json!({"balance": balance})))
  }
  else {
    Err(ConnectorError::OtherError("Transaction failed".to_owned()))
  }
}

/// Given an offering address, read the balance owned by the 
#[get("/delegated/dt/{nft_address}")]
async fn get_dt(  
  path: web::Path<String>,
  iota_state: web::Data<IotaState>,
  sc_provider: web::Data<ScProvider>)
    -> Result<impl Responder, ConnectorError>
  {

  // Retrieve the data token address associated to the offering
  let nft_address = Address::from_str(&path)?;
  let servicebase = ServiceBase::new(nft_address, sc_provider.clone().into_inner());

  let dt_address = servicebase
    .getATaddresses()
    .call()
    .await
    .ok()
    .and_then(|addresses| addresses.first().cloned())
    .ok_or(ConnectorError::OtherError("DT address not found".to_owned()))?;

  let access_token_base_contract = AccessTokenBase::new(dt_address, sc_provider.into_inner());
  
  let booth_address = iota_state.get_evm_address()
    .await
    .and_then(|booth_addr| Ok(Address::from_str(&booth_addr)?))?;

  let balance: u32 = access_token_base_contract.balanceOf(booth_address)
    .call().await
    .map(|b| b / Unit::ETHER.wei_const())
    .map_err(|e| ConnectorError::OtherError(format!("Contract reverted: {}", e.to_string())))?
    .to();

  Ok(HttpResponse::Ok().json(json!({"balance": balance})))
}
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
      .service(buy_dt)
      .service(get_dt);
}