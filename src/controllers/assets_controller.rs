// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use std::fs::File;

use actix_web::{get, patch, post, Responder};
use actix_web::{web, HttpResponse};
use deadpool_postgres::Pool;
use identity_iota::storage::{JwkDocumentExt, JwsSignatureOptions};
use ipfs_api_backend_actix::{IpfsClient, IpfsApi};
use serde_json::json;

use crate::BASE_UPLOADS_DIR;
use crate::errors::ConnectorError;
use crate::dtos::{UploadForm, AssetUploadRequest, QueryEthAddress, QueryAssetAlias, AssetUpdateRequest};
use crate::models::asset::Asset;
use crate::repository::asset_operations::AssetExt;
use crate::repository::identity_operations::IdentityExt;
use crate::utils::iota::IotaState;
use actix_multipart::form::MultipartForm;
use uuid::Uuid;
use blake2::{Blake2b512, Blake2s256, Digest};
use base64::{Engine as _, engine::general_purpose};


#[post("/assets")] // TODO: improve request size 
async fn upload_asset(
    MultipartForm(form): MultipartForm<UploadForm>,
    ipfs_client: web::Data<IpfsClient>,
    db_pool: web::Data<Pool>,
    iota_state: web::Data<IotaState>
) -> Result<impl Responder, ConnectorError> {
    
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;   
    let identity = pg_client.get_identity_with_eth_addr(&form.eth_address).await?;
    // TODO: simplify content of this function
    let files = form.files;

    if files.len() != 2 {
        return Err(ConnectorError::FileUploadError)
    }

    let mut offering_path_option = None;
    let mut asset_path_option = None;

    for f in files {
        let file_name = f.file_name.ok_or(ConnectorError::FileNameMissing)?;
        let path = format!("{}{}-{}", BASE_UPLOADS_DIR, Uuid::new_v4().to_string(), file_name.clone() );
        log::info!("Saving {}", path);

        if file_name.contains("offering") {
            offering_path_option = Some(path.clone());
        } else if file_name.contains("asset") {
            asset_path_option = Some(path.clone());
        }
        
        f.file.persist(path).map_err(|_| ConnectorError::PersistFileError )?;
    }
    
    let offering_path = offering_path_option.ok_or(ConnectorError::FileNameMissing)?;
    let asset_path = asset_path_option.ok_or(ConnectorError::FileNameMissing)?;

    let mut offering_file = File::open(&offering_path)?;
    let mut asset_file = File::open(&asset_path)?;

    let mut hasher = Blake2b512::new();
    let _ = std::io::copy(&mut asset_file, &mut hasher)?;
    let asset_file_hash = hasher.finalize_reset();
    let _ = std::io::copy(&mut offering_file, &mut hasher)?;
    let offering_file_hash = hasher.finalize();

    let document = iota_state.resolve_did(identity.did.as_str()).await?;

    let base64_asset_hash = general_purpose::STANDARD.encode(&asset_file_hash.as_slice());
    let base64_offering_hash = general_purpose::STANDARD.encode(&offering_file_hash.as_slice());

    let payload = json!({"assetHash": base64_asset_hash, "offeringHash": base64_offering_hash});
    log::info!("payload: {:#?}", payload);
    let jws = document.create_jws(
        &iota_state.storage,
        &identity.fragment, 
        serde_json::to_string(&payload)?.as_bytes(),
        &JwsSignatureOptions::default()
    ).await?;
    log::info!("jws: {:#?}", jws);

    // load offering file's content of IPFS and get CID back
    let ipfs_response =  ipfs_client.add(offering_file).await?;
    log::info!("cid: {}", ipfs_response.hash);
    
    let new_asset = Asset { 
        id: None,
        nft_address: None, 
        cid: ipfs_response.hash, 
        alias: form.alias.clone(), 
        asset_path: asset_path, 
        offering_path: offering_path, 
        asset_hash: base64_asset_hash, 
        offering_hash: base64_offering_hash, 
        sign: jws.into(), 
        publisher: identity.id.ok_or(ConnectorError::IdMissing)?
    };
    let asset = pg_client.insert_asset_info(&new_asset).await?;

    Ok(HttpResponse::Ok().json(asset))
}

#[get("/assets/aliases")]
async fn get_asset_aliases(
    query_params: web::Query<QueryEthAddress>, 
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller get_asset_aliases");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let res = pg_client.get_aliases(&query_params.eth_address).await?;
    Ok(HttpResponse::Ok().json(json!({"aliases": res})))
}

#[get("/assets")]
async fn get_asset_info_from_alias(
    query_params: web::Query<QueryAssetAlias>, 
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller get_asset_info_from_alias");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let asset = pg_client.get_asset_info_from_alias(&query_params.alias).await?;

    Ok(HttpResponse::Ok().json(asset))
}

#[get("/assets/{asset_id}")]
async fn get_asset_info(
    path: web::Path<i64>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller get_asset_info");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let asset_id = path.into_inner();    
    let asset = pg_client.get_asset_info(asset_id).await?;

    Ok(HttpResponse::Ok().json(asset))
}

/// update the nft address
// #[patch("/assets/{asset_id}")]
#[patch("/assets")]
async fn patch_asset(
    // path: web::Path<i64>,
    query_params: web::Query<QueryAssetAlias>, 
    req_body: web::Json<AssetUpdateRequest>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller patch_asset");
    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let asset = pg_client.set_nft_address(&query_params.alias, &req_body.nft_address).await?;

    Ok(HttpResponse::Ok().json(asset))
}

#[post("/assets/{asset_id}/challenge")]
async fn get_asset_challenge(
    path: web::Path<i64>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller get_asset_challenge");
    todo!()
}

#[post("/assets/{asset_id}/download")]
async fn download_asset( // todo: this should be a protected route
    path: web::Path<i64>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller download_asset");
    todo!()
}

#[get("/assets/{asset_id}/encrypt-cid")]
async fn encrypt_asset_cid(
    path: web::Path<i64>,
    db_pool: web::Data<Pool>,
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller encrypt_asset_cid");
    todo!()
}


// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg
    .service(get_asset_aliases)     
    .service(upload_asset)
    .service(get_asset_info_from_alias)   
    .service(get_asset_info)       
    .service(patch_asset);
    // .service(get_asset_challenge)
    // .service(download_asset)
    // .service(encrypt_asset_cid)
}