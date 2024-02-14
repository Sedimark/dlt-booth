// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use actix_web::get;
use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use identity_iota::core::{Timestamp, Duration};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use crate::models::download_request::{self, DownloadRequest};
use crate::repository::download_request_operations::ChallengesExt;

use crate::errors::ConnectorError;

use crate::dtos::ChallengeResponse;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Params {
    did: String,
}

#[get("/challenges")]
async fn get_challenge(
    params: web::Query<Params>, 
    db_pool: web::Data<Pool>,
) -> Result<impl Responder, ConnectorError> {
    log::info!("get_challenge");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    log::info!("{}", params.did);
    // check and clean holder requests
    match pg_client.get_challenge(&params.did).await {
        Ok(challenge) => {
            // request already exists
            // check that it is not expired, if expired remove from db
            let holder_request_timestamp = Timestamp::parse(&challenge.expiration).unwrap();
            if holder_request_timestamp < Timestamp::now_utc() {
               // request expired --> remove it from DB and let handler continue
               pg_client.remove_challenge(&challenge.nonce).await?;
            } else {
               // request still not expired --> stop handler from continuing
                return Err(ConnectorError::ChallengePendingError)
            }
        },
        Err(ConnectorError::RowNotFound) => {log::info!("No pending challenges");},
        Err(err) => return Err(err),
    }

    // create nonce and store holder request (did, request expiration, nonce)
    let expiration = Timestamp::now_utc().checked_add(Duration::minutes(1)).unwrap();
    // let nonce = "0x".to_owned() + &Uuid::new_v4().simple().to_string();
    let nonce = Uuid::new_v4().to_string();

    let download_request = DownloadRequest {
        nonce: nonce.clone(),
        requester_did: params.did.clone(),
        expiration: expiration.to_string(),
    };

    log::info!("Download request: {:?}", download_request);
    pg_client.insert_challenge(&download_request).await?;
    
    Ok(HttpResponse::Ok().json(ChallengeResponse {nonce: nonce.clone()}))
}

// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(get_challenge);
}