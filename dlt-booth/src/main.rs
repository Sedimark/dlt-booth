// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::str::FromStr;

use actix_web::{http::{self}, middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use alloy::providers::ProviderBuilder;
use dlt_booth::{contracts::ScProvider, controllers, repository::postgres_repo::init, utils::{configs::{DLTConfig, DatabaseConfig, EvmAddressConfig, HttpServerConfig, KeyStorageConfig, WalletStorageConfig}, iota::IotaState, issuer::Issuer}};
use clap::Parser;
use url::Url;

/// DLT Booth command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    
    /// Configuration parameters for the DLT
    #[command(flatten)]
    dlt_config: DLTConfig,

    /// HTTP Server configuration
    #[command(flatten)]
    http_server_config: HttpServerConfig,

    /// Configuration section for the KeyStorage
    #[command(flatten)]
    key_storage_config: KeyStorageConfig,

    /// Configuration for the Wallet
    #[command(flatten)]
    wallet_config: WalletStorageConfig,

    /// Database configuration args
    #[command(flatten)]
    database_config: DatabaseConfig
}

#[actix_web::main]
async fn main() -> anyhow::Result<()>{
    #[cfg(debug_assertions)]
    dotenv::from_path("./env/.env").expect(".env file not found");
    #[cfg(debug_assertions)]
    dotenv::from_path("./env/postgres.env").expect("postgres.env file not found");

    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    let address = args.http_server_config.host_address;
    let port = args.http_server_config.host_port;

    // Initialize database connection pool
    let db_pool = init(args.database_config).await?;

    // Initialize provider
    let rpc_provider =  args.dlt_config.rpc_provider.clone(); 

    let issuer_url = &args.dlt_config.issuer_url.clone();
    // Initialize iota (wallet, client, etc.)
    let evm_config = EvmAddressConfig::default();

    let iota_state = IotaState::init(args.key_storage_config, args.wallet_config, args.dlt_config, evm_config).await?;
    let iota_state_data = web::Data::new(iota_state);

    let issuer_client = Issuer::init(issuer_url)?;

    log::info!("Initializing custom provider");
    let provider: ScProvider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(Url::from_str(&rpc_provider)?);
    
    let provider_data = web::Data::new(provider);

    log::info!("Starting up on {}:{}", address, port);
    HttpServer::new(move || {
        let cors = Cors::default()
        .allow_any_origin() // TODO: define who is allowed (just frontend?)
        .allowed_methods(vec!["GET", "POST", "PATCH"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .max_age(3600);

        App::new()
        .app_data(web::Data::new(db_pool.clone()))
        .app_data(iota_state_data.clone())
        .app_data(provider_data.clone())
        .app_data(web::Data::new(issuer_client.clone()))
        .service(web::scope("/api")
            //.configure(controllers::identities_controller::scoped_config)
            //.configure(controllers::challenges_controller::scoped_config)
            .configure(controllers::delegated_identities::scoped_config)
            .configure(controllers::dids_controller::scoped_config)
        )
        .wrap(cors)
        .wrap(Logger::default())
    })
    .bind((address, port))?
    .run()
    .await.map_err(anyhow::Error::from)
}