// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use actix_web::{get, web, HttpResponse};

use crate::{errors::ConnectorError, utils::iota::IotaState};

/// Resolve DID endpoint
#[get("/dids/{did}")]
async fn resolve_did(
  path: web::Path<String>,
  iota_state: web::Data<IotaState>
) -> Result<HttpResponse, ConnectorError> {
  let did = path.into_inner();

  let document = iota_state.resolve_did(&did).await?;

  Ok(HttpResponse::Ok().json(document))
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
  cfg
  .service(resolve_did);
}