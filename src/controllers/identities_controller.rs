// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_web::{get, patch, post};
use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use identity_iota::storage::key_storage;
use serde_json::json;

use crate::errors::ConnectorError;
use crate::services::identity_service;

#[post("")] 
async fn create_identity(
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    identity_service::create_identity(&pg_client, "hello".to_string()).await?;

    Ok(HttpResponse::Ok().json(json!({"hello" : "word"})))
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