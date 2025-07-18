// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ops::Deref;

use actix_multipart::form::{bytes::Bytes, MultipartForm};
use actix_web::{body::MessageBody, get, post, web, HttpResponse, Responder};
use identity_iota::{did::{DIDUrl, DID}, verification::jwu::decode_b64};
use serde::Deserialize;
use serde_json::json;
use url::Url;

use crate::{errors::ConnectorError, utils::iota::IotaState};

#[derive(Debug, Deserialize)]
struct VerifySignatureRequest{
    message: String,
    signature: String
}

#[derive(Deserialize)]
struct DidQuery{ did: Url}

/// Resolve DID endpoint
#[get("/dids")]
async fn resolve_did(
  did: web::Query<DidQuery>,
  iota_state: web::Data<IotaState>
) -> Result<HttpResponse, ConnectorError> {
  let did = did.did.as_str();
  
  let document = iota_state.resolve_did(did).await?;

  Ok(HttpResponse::Ok().json(document.core_document()))
}

#[post("/dids/verify")]
async fn verify_signature(
  verify_request: actix_web::web::Json<VerifySignatureRequest>,
  did: web::Query<DidQuery>,
  iota_state: web::Data<IotaState>
) -> Result<impl Responder, ConnectorError>
{

  // parse DIDUrl
  let did_url = DIDUrl::parse(did.did.as_str())
    .map_err(|_| ConnectorError::OtherError("Not a valid did url".to_string()))?;

  // resolve DID
  let document = iota_state.resolve_did(did_url.did().as_str())
    .await?;

  let signature = decode_b64( verify_request.signature.clone())
    .map(|decoded| decoded.into_boxed_slice()).ok();
  let message  = decode_b64(verify_request.message.clone())
    .map(|decoded| decoded.into_boxed_slice()).ok();

  if signature.as_ref().or(message.as_ref()).is_none(){
    Ok(HttpResponse::BadRequest().json(json!({"message": "Cannot decode data"})))
  }
  else{
    //checked in the if clause, safe unwrap
    iota_state.verify_signature(did_url, document, signature.unwrap(), message.unwrap())?;
    Ok(HttpResponse::Ok().finish())
  }

}
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
  cfg
  .service(resolve_did)
  .service(verify_signature);
}