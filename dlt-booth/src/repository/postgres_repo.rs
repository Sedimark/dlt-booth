// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use anyhow::Result;

use deadpool_postgres::{ManagerConfig, RecyclingMethod, Pool};
use tokio_postgres::NoTls;

use crate::utils::configs::DatabaseConfig;

use super::download_request_operations::ChallengesExt;

/// Clean challenges from the database every hour
async fn cleanup_loop(pool: Pool)
{   
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    if let Ok(client) = pool.get().await {
        loop
        {
            // cleanup loop
            interval.tick().await;
            let result = client.cleanup_challenge().await;
            match result {
                Ok(_) => log::info!("SQL cleanup completed"),
                Err(err) => log::error!("SQL cleanup error: {}", err.to_string())
            }
        }       
    }
    else {
        log::error!("Cannot access DB pool. Cleanup disabled");
    }

}

pub async fn init(db_config: DatabaseConfig) -> Result<Pool> {
    log::info!("init database");

    let mut config = deadpool_postgres::Config::new();
    config.user = Some(db_config.db_user);
    config.password = Some(db_config.db_password.value());
    config.dbname = Some(db_config.db_name);
    config.host = Some(db_config.db_host);
    config.port = Some(db_config.db_port);

    config.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
    let pool = config.create_pool(None, NoTls)?;
    log::info!("pool database");

    tokio::task::spawn(cleanup_loop(pool.clone()));
    Ok(pool)
}   