# DLT Booth

## ToC

1. [Environnement setup](#environnement-setup)
2. [Running the application](#running-the-application)
3. [Useful tools for development and integration](#dev-utils)

## Environnement setup
In connector-rs/env folder, create a .env file starting from .env.example and update the values accordingly to your development enviroment.

```env
RUST_LOG=debug # Log level [debug, info, error]

# HTTP SERVER CONFIG
HOST_ADDRESS=0.0.0.0 # Http server bind address
HOST_PORT=8085 # Http server bind port

# DLT CONFIG
NODE_URL="https://api.testnet.shimmer.network"
FAUCET_API_ENDPOINT="https://faucet.testnet.shimmer.network/api/enqueue"
RPC_PROVIDER="https://json-rpc.evm.testnet.shimmer.network"
CHAIN_ID=1073

# KEY STORAGE CONFIG
KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH="./key_storage.stronghold" # file path where secrets will be stored
KEY_STORAGE_STRONGHOLD_PASSWORD="some_hopefully_secure_password" # password used for Stronghold file encryption
KEY_STORAGE_MNEMONIC="grace eye hour away retire put crunch burger bracket coyote twist cherry glare collect retreat" # mnemonic seed for key storage generation

## WALLET CONFIG
WALLET_STRONGHOLD_SNAPSHOT_PATH="./wallet.stronghold" # file path where wallet keys will be stored
WALLET_STRONGHOLD_PASSWORD="some_hopefully_secure_password" # password used for Stronghold file encryption
WALLET_MNEMONIC="grace eye hour away retire put crunch burger bracket coyote twist cherry glare collect retreat" # mnemonic seed for wallet generation

# DATABASE CONNECTION CONFIG
DB_USER="postgres"
DB_PASSWORD="connector"
DB_NAME="connector"
DB_HOST=127.0.0.1 # "postgres" for deploying, "127.0.0.1 " for dev
DB_PORT=5432
DB_MAX_POOL_SIZE=16

```
## Running the Application

### Running from Docker

A Docker compose file has been set for deploying. A Postgres database and the application container is created.

```bash
docker compose --profile deploy up -d
```

Remember to configure environnment variables properly before running the command.


## Dev Utils
- [OpenAPI spec](/api/dlt_booth.yaml)
- [Bruno APIs](/api/dlt-booth-api)
## License

[GPL-3.0-or-later](https://spdx.org/licenses/GPL-3.0-or-later.html)