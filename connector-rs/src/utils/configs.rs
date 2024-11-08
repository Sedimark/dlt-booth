// SPDX-FileCopyrightText: 2024 Fondazione LINKS
//
// SPDX-License-Identifier: GPL-3.0-or-later

use zeroize::ZeroizeOnDrop;
use clap::Args;

/// Simple configuration of a generic secret read from Args.
/// Must be deleted when it is not needed anymore
#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct ConfigSecret(String);

impl std::str::FromStr for ConfigSecret {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl ConfigSecret {
    pub fn value(&self) -> String {
        self.0.clone()
    }
}

/// Configuration parameters for the key storage
#[derive(Args, Debug)]
pub struct KeyStorageConfig {
    /// File path for the KeyStorage
    #[arg(
        id = "KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH",
        long,
        env,
        required = true
    )]
    pub file_path: String,
    /// Secrets that unlocks the KeyStorage
    #[arg(id = "KEY_STORAGE_STRONGHOLD_PASSWORD", long, env, required = true)]
    pub password: ConfigSecret,
    /// Mnemonic seed to be stored inside the KeyStorage
    #[arg(id = "KEY_STORAGE_MNEMONIC", long, env, required = true)]
    pub mnemonic: ConfigSecret,
}

/// Configuration parameters for the key storage
#[derive(Args, Debug)]
pub struct WalletStorageConfig {
    /// File path for the KeyStorage
    #[arg(
        id = "WALLET_STRONGHOLD_SNAPSHOT_PATH",
        long,
        env,
        required = true
    )]
    pub file_path: String,
    /// Secrets that unlocks the KeyStorage
    #[arg(id = "WALLET_STRONGHOLD_PASSWORD", long, env, required = true)]
    pub password: ConfigSecret,
    /// Mnemonic seed to be stored inside the KeyStorage
    #[arg(id = "WALLET_MNEMONIC", long, env, required = true)]
    pub mnemonic: ConfigSecret,
}

/// Configuration parameters for the issuer database
#[derive(Args, Debug)]
pub struct DatabaseConfig {
    /// Postgres host address
    #[arg(long, env, required = true)]
    pub db_host: String,
    /// Postgres db port
    #[arg(long, env, required = true)]
    pub db_port: u16,
    /// Postgres db name
    #[arg(long, env, required = true)]
    pub db_name: String,
    /// Postgres username
    #[arg(long, env, required = true)]
    pub db_user: String,
    /// Postgres password
    #[arg(long, env, required = true)]
    pub db_password: ConfigSecret,
    /// Postgres max pool size
    #[arg(long, env, default_value_t = 5432)]
    pub db_max_pool_size: u16,
}

/// Configuration for_ the http server
#[derive(Args, Debug)]
pub struct HttpServerConfig {
    /// Bind address for the http server
    #[arg(long, env, required = true)]
    pub host_address: String,

    /// Listening port for the http server
    #[arg(long, env, default_value_t = 8085)]
    pub host_port: u16,
}

/// Configuration of the verifiable data registry
#[derive(Debug, Args)]
pub struct DLTConfig {
    /// JSON RPC provider url
    #[arg(long, env, required = true)]
    pub rpc_provider: String,

    /// Chain id
    #[arg(long, env, required = true)]
    pub chain_id: u64,

    /// URL for reaching the DLT node
    #[arg(long, env, required = true)]
    pub node_url: String,

    /// Faucet API endpoint
    #[arg(long, env, required = true)]
    pub faucet_api_endpoint: String,

    /// Explorer Url
    #[arg(long, env, required = true)]   
    pub explorer_url: String,    
    
    /// Issuer Endpoint
    #[arg(long, env, required = true)]   
    pub issuer_url: String
}