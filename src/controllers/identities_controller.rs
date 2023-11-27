// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_web::{get, patch, post};
use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use identity_iota::storage::key_storage;
use serde_json::json;

use crate::dtos::IdentityRequest;
use crate::errors::ConnectorError;
use crate::models::identity::Identity;
use crate::repository::identity_operations::IdentityExt;
use crate::services::identity_service;
use crate::utils::iota::IotaState;

#[post("")] 
async fn create_identity(
    req_body: web::Json<IdentityRequest>, 
    db_pool: web::Data<Pool>,
    iota_state: web::Data<IotaState>
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let (doc, fragment) = iota_state.create_did().await?;

    let new_identity = Identity {
        eth_address: req_body.eth_address.clone(),
        did: doc.id().to_string(),
        fragment: fragment,
        vcredential: None,
    };
    let _ = pg_client.insert_identity(&new_identity).await?;

    Ok(HttpResponse::Ok().json(new_identity))
}

#[get("/{eth_address}")]
async fn get_identity(
    path: web::Path<String>
) -> Result<HttpResponse, ConnectorError> {
    todo!();
}

#[patch("/{eth_address}")]
async fn patch_identity(
    path: web::Path<String>
) -> Result<HttpResponse, ConnectorError> {
    todo!();
}

// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        // prefixes all resources and routes attached to it...
        web::scope("/identities")
        .service(create_identity)
        .service(get_identity)     
        .service(patch_identity)       
    );
}