// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_web::{get, patch, post};
use actix_web::{web, HttpResponse, Responder};
use identity_iota::storage::key_storage;

use crate::errors::ConnectorError;


#[post("")] 
async fn upload_asset(
) -> Result<HttpResponse, ConnectorError> {
    todo!();
}

#[get("")]
async fn get_assets_info(
    path: web::Path<String>
) -> Result<HttpResponse, ConnectorError> {
    todo!();
}

#[patch("/{asset_id}")]
async fn get_asset_info(
    path: web::Path<String>
) -> Result<HttpResponse, ConnectorError> {
    todo!();
}

// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        // prefixes all resources and routes attached to it...
        web::scope("/assets")
        .service(upload_asset)
        .service(get_assets_info)     
        .service(get_asset_info)       
    );
}