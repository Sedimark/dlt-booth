// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::io::Seek;
use std::str::FromStr;
use std::sync::Arc;

use actix_web::web::ReqData;
use actix_web::{get, patch, post, HttpMessage, HttpRequest, Responder};
use actix_web::{web, HttpResponse};
use deadpool_postgres::Pool;
use ethers::abi::Address;
use ethers::providers::{Http, Provider};
use ethers::types::{Bytes, Signature};
use futures_util::TryStreamExt;
use hex::FromHex;
use identity_iota::core::ToJson;
use identity_iota::storage::{JwkDocumentExt, JwsSignatureOptions};
use ipfs_api_backend_actix::{IpfsClient, IpfsApi};
use serde_json::json;

use actix_web_lab::middleware::from_fn;
use crate::middlewares::ver_presentation_jwt::verify_presentation_jwt;

use crate::BASE_UPLOADS_DIR;
use crate::errors::ConnectorError;
use crate::dtos::{AssetUpdateRequest, ProofOfPurchaseRequest, QueryAssetAlias, QueryEthAddress, UploadForm};
use crate::models::asset::Asset;
use crate::repository::asset_operations::AssetExt;
use crate::repository::identity_operations::IdentityExt;
use crate::utils::iota::IotaState;
use actix_multipart::form::MultipartForm;
use uuid::Uuid;
use blake2::{Blake2b512, Digest};
use base64::{Engine as _, engine::general_purpose};
use crate::contracts::servicebase::ServiceBase;

#[get("/cids/{cid}")] // TODO: improve request size 
async fn get_description_from_ipfs(
    path: web::Path<String>,
    ipfs_client: web::Data<IpfsClient>,
) -> Result<impl Responder, ConnectorError> {
    
    let cid = path.into_inner();    
    // get offering file's content from IPFS
    let res = ipfs_client
    .cat(cid.as_ref())
    .map_ok(|chunk| chunk.to_vec())
    .try_concat()
    .await?;

    Ok(HttpResponse::Ok().body(res))
}

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

    // TODO: remove this 
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
        &iota_state.key_storage,
        &identity.fragment, 
        serde_json::to_string(&payload)?.as_bytes(),
        &JwsSignatureOptions::default()
    ).await?;
    log::info!("jws: {:#?}", jws);

    // load offering file's content of IPFS and get CID back
    offering_file.rewind()?;
    let ipfs_response =  ipfs_client.add(offering_file).await?;
    log::info!("cid: {}", ipfs_response.hash);
    
    let new_asset = Asset { 
        id: None,
        nft_address: None, 
        cid: ipfs_response.hash, 
        alias: form.alias.clone(), 
        asset_path: asset_path, 
        offering_path: offering_path, 
        asset_hash: base64_asset_hash,  // TODO: remove this, also on the front-end
        offering_hash: base64_offering_hash,  // TODO: remove this, also on the front-end
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
    let asset_info = pg_client.get_asset_info(asset_id).await?;

    Ok(HttpResponse::Ok().json(asset_info))
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


#[get("/assets/download", wrap = "from_fn(verify_presentation_jwt)")]
async fn download_asset(
    req: HttpRequest,
    query_params: web::Query<QueryAssetAlias>, 
    db_pool: web::Data<Pool>,
    eth_provider: web::Data<Arc<Provider<Http>>>,
    opt_pop_req: Option<ReqData<ProofOfPurchaseRequest>>,
) -> Result<HttpResponse, ConnectorError> {
    log::info!("controller download_asset");

    let pg_client = db_pool.get().await.map_err(ConnectorError::PoolError)?;
    let asset_info = pg_client.get_asset_info_from_alias(&query_params.alias).await?;

    let pop_req = opt_pop_req.ok_or(ConnectorError::MiddlewareError("Missing Proof of purchase request".to_string()))?.into_inner();
    log::info!("{:#?},", pop_req);

    // Verify proof of purchase
    let address: Address = asset_info.nft_address.ok_or(ConnectorError::NftAddressMissing)?.parse().map_err(|_| ConnectorError::AddressRecoveryError)?;
    let client = eth_provider.get_ref().clone();
    let contract = ServiceBase::new(address, client);

    // Convert the signature and nonce to bytes
    let eth_sig_bytes = Bytes::from(Vec::from_hex(pop_req.eth_signature.strip_prefix("0x").ok_or(ConnectorError::StringToBytesError)?).map_err(|_| ConnectorError::StringToBytesError)?);
    let nonce_bytes = Bytes::from(pop_req.nonce.into_bytes());

    // Call verifyPoP on the contract
    let pop = contract.verify_proof_of_purchase(eth_sig_bytes, nonce_bytes).await.map_err(|_| ConnectorError::ContractError)?;

    if pop == false {
        log::info!("Proof of purchase failed, user not allower to download asset");
        return Ok(HttpResponse::Unauthorized().json(json!({"error": "Proof of purchase failed"})));
    }
    
    // Return the file
    let file = actix_files::NamedFile::open_async(asset_info.asset_path).await?;
    Ok(file.into_response(&req))

}

// this function could be located in a different module
pub fn scoped_config(cfg: &mut web::ServiceConfig) {

    // order: first come first served

    cfg
    .service(get_asset_aliases)     
    .service(upload_asset)
    .service(get_asset_info_from_alias)   
    .service(download_asset)       
    .service(patch_asset)
    .service(get_asset_info)
    .service(get_description_from_ipfs);
}