// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use actix_web::{http::{self}, middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use connector::{controllers, utils::iota::IotaState, repository::postgres_repo::init, BASE_UPLOADS_DIR};
use ethers::providers::{Http, Provider};
use ipfs_api_backend_actix::{IpfsClient, TryFromUri};
use clap::Parser;

/// Connector command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// JSON RPC provider url
    #[arg(long, env, required=true)]
    rpc_provider: String,

    /// chain id
    #[arg(long, env, required=true)]
    chain_id: u64,

    /// Ipfs client api url
    #[arg(long, env, required=true)]
    ipfs_url: url::Url
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    dotenv::from_path("./env/.env").expect(".env file not found");
    #[cfg(debug_assertions)]
    dotenv::from_path("./env/postgres.env").expect("postgres.env file not found");

    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    let address = std::env::var("ADDR").expect("$ADDR must be set.");
    let port = std::env::var("PORT").expect("$PORT must be set.").parse::<u16>()?;

    // Initialize database connection pool
    let db_pool = init().await?;

    // Initialize iota (wallet, client, etc.)
    let iota_state = IotaState::init().await?;
    let iota_state_data = web::Data::new(iota_state);
    
    // Initialize provider
    let rpc_provider =  args.rpc_provider; 
    let _chain_id = args.chain_id;

    log::info!("Initializing custom provider");
    let provider = Arc::new(Provider::<Http>::try_from(rpc_provider)?);
    let provider_data = web::Data::new(provider);


    // Create uploads directory if it doesn't exist
    std::fs::create_dir_all(BASE_UPLOADS_DIR)?;

    log::info!("Starting up on {}:{}", address, port);
    HttpServer::new(move || {
        let cors = Cors::default()
        .allow_any_origin() // TODO: define who is allowed (just frontend?)
        .allowed_methods(vec!["GET", "POST", "PATCH"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600);
        
        let ipfs_client = IpfsClient::from_str("http://127.0.0.1:5001")
            .unwrap(); // This may let crash the app. TODO: there must be a better solution.

        App::new()
        .app_data(web::Data::new(ipfs_client)) // connect to the default IPFS API address http://localhost:5001
        .app_data(web::Data::new(db_pool.clone()))
        .app_data(iota_state_data.clone())
        .app_data(provider_data.clone())
        .service(web::scope("/api")
            .configure(controllers::identities_controller::scoped_config)
            .configure(controllers::assets_controller::scoped_config)
            .configure(controllers::challenges_controller::scoped_config)
        )
        .wrap(cors)
        .wrap(Logger::default())
    })
    .bind((address, port))?
    .run()
    .await.map_err(anyhow::Error::from)
}