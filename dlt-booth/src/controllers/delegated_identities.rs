// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use actix_web::{delete, post};
use actix_web::{web, HttpResponse};
use deadpool_postgres::Pool;
use identity_iota::iota::IotaDID;
use serde_json::json;
use crate::errors::ConnectorError;
use crate::models::identity::Identity;
use crate::repository::identity_operations::IdentityExt;
use crate::utils::iota::IotaState;
use crate::utils::issuer::Issuer;


#[post("/identities")] 
async fn create_identity(
    req_body: web::Json<crate::dtos::Identity>,
    db_pool: web::Data<Pool>,
    iota_state: web::Data<IotaState>,
    issuer_client: web::Data<Issuer>
) -> Result<HttpResponse, ConnectorError> {
    log::debug!("controller create_identity");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let eth_address = iota_state.get_evm_address().await?;

    // Check if an identity already exists 
    // If an identity already exists, it should not be possible to create a new one without revoking the old one
    let new_identity = match pg_client.get_identity_with_eth_addr(&eth_address).await{
        Err(ConnectorError::RowNotFound) => {
            // create a new DID document and publish on L1
            let (doc, fragment) = iota_state.create_did(Some(eth_address.as_str()), req_body.services.clone()).await?;

            let new_identity = Identity {
                id: None,
                eth_address: eth_address.to_owned(),
                did: doc.id().to_string(),
                fragment: fragment,
                vcredential: None,
            };

            pg_client.insert_identity(&new_identity).await?

        },
        Ok(identity) => {
            // if a VC has already been issued then do not continue
            identity
        }
        Err(e) => return Err(e) 
    };

    // Check if VC already exists
    if let Some(_) = new_identity.vcredential {
        return Ok(HttpResponse::Forbidden().json(json!({"message": "Cannot create a VC twice"})))
    }

    // create a new VC and register by the Issuer
    log::info!("Requesting a VC for {}", new_identity.did);
    let created_identity = issuer_client.register(&new_identity, &iota_state, req_body.0.credential).await?;
    let updated_identity = pg_client.set_credential(&created_identity.eth_address, &created_identity.vcredential).await?;
    log::info!("Vc saved!");
    
    match updated_identity.vcredential {
        Some(vc) => Ok(HttpResponse::Ok().json(json!({"credential": vc}))),
        None => Ok(HttpResponse::InternalServerError().json(json!({"message": "Unexpected error when reading VC"})))
    }

}

#[delete("/identities")]
async fn delete_identity(
    db_pool: web::Data<Pool>,
    iota_state: web::Data<IotaState>,
)-> Result<HttpResponse, ConnectorError>{
    log::debug!("controller delete_identity");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let eth_address = iota_state.get_evm_address().await?;

    let identity = pg_client.get_identity_with_eth_addr(&eth_address).await?;

    // Credential deleted. Now delete it from DLT
    let did = IotaDID::parse(identity.did.as_str())?;
    iota_state.delete_did(&did).await?;

    // Finally drop the DID from database
    pg_client.delete_identity(&identity.did).await?;

    Ok(HttpResponse::Ok().finish())
}
// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(web::scope("/delegated")
        .service(create_identity)
        .service(delete_identity)
    );
    //.service(get_identity)     
    //.service(patch_identity)       
    //.service(sign_data)
    //.service(gen_presentation);
}