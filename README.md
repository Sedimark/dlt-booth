# DLT Booth

DLT Booth is a fork of the [connector component of the Mediterraneus Protocol](https://github.com/Cybersecurity-LINKS/mediterraneus-connector). This component has been customized to be integrated into the Sedimark Toolbox.

DLT Booth consists in an http server written in Rust and deployed as a docker image, which requires to be connected to a Postgres database. 

### Main functionalities

DLT Booth can be used to execute operation on a configured EVM through http requests. It also communicates with a configured DLT to perform SSI operations:

**SSI operations**
- Create a DID Document
- Publish a DID Document on the configured DLT
- Request for Verifiable Credential issuance to a Issuer for Sedimark marketplace onboarding
- Request for Verifiable Credential revocation
- Create Verifiable Presentation

**EVM operations**
- EIP191 formatted message signatures
- Execute EVM transactions

## ToC

1. [Environnement setup](#environnement-setup)
2. [Running the application](#running-the-application)
3. [Useful tools for development and integration](#dev-utils)

## Environment setup
In dlt-booth/env folder, create a .env file starting from example.env and update the values accordingly to your development enviroment.

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

# ISSUER CONFIG
ISSUER_URL=http://localhost:3213/api

# KEY STORAGE CONFIG
KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH="./key_storage.stronghold" # file path where secrets will be stored
KEY_STORAGE_STRONGHOLD_PASSWORD="some_hopefully_secure_password" # password used for Stronghold file encryption

## WALLET CONFIG
WALLET_STRONGHOLD_SNAPSHOT_PATH="./wallet.stronghold" # file path where wallet keys will be stored
WALLET_STRONGHOLD_PASSWORD="some_hopefully_secure_password" # password used for Stronghold file encryption

# DATABASE CONNECTION CONFIG
DB_USER="postgres"
DB_PASSWORD="dlt_booth"
DB_NAME="dlt_booth"
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
Remember to properly configure [environnment variables](#environment-setup)  before running the command.

#### Backup and restore
Generated keys are stored in files generated for key storage and wallet. They're both saved in `/data` inside the container. Those files are protected with the passwords provided in the [environnment variables](#environment-setup). The data folder needs to be syncronized with postgres data as well.

##### Backup operation
- Copy the container `/data` folder into a different storage
- Store the passwords securely
- Backup Postgres

##### Restore operation
- Copy back into the `/data` folder
- Set corresponding passwords in the environnment variable
- Restore Postgres backup

## Dev Utils
- [OpenAPI spec](/api/dlt_booth.yaml)
- [Bruno APIs](/api/dlt-booth-api)

### Docker compose example
```yaml
services:
  dlt-booth:
    build:
      context: .
      dockerfile: Dockerfile
    image: dlt-booth
    container_name: dlt-booth
    restart: unless-stopped
    ports:
      - "8085:8085"
    depends_on:
      postgres:
       condition: service_healthy 
    #enables data persistance of key storage and wallet. Remove it to disable data persistance
    volumes:
      - "./docker_data:/data"
    networks:
      - dlt-booth-net
    environment:
      RUST_BACKTRACE: 1
      RUST_LOG: info
      # HTTP SERVER CONFIG
      HOST_ADDRESS: 0.0.0.0
      HOST_PORT: 8085
      # DLT CONFIG
      NODE_URL: https://stardust.unican.sedimark.eu
      FAUCET_API_ENDPOINT: https://faucet.tangle.stardust.linksfoundation.com/api/enqueue
      RPC_PROVIDER: https://stardust.unican.sedimark.eu/sedimark-chain
      CHAIN_ID: 1074
      # ISSUER CONFIG
      ISSUER_URL: http://sedimark-issuer-rs:3213/api
      # KEY STORAGE CONFIG
      KEY_STORAGE_STRONGHOLD_SNAPSHOT_PATH: ./key_storage.stronghold
      KEY_STORAGE_STRONGHOLD_PASSWORD: some_hopefully_secure_password
      # WALLET CONFIG
      WALLET_STRONGHOLD_SNAPSHOT_PATH: ./wallet.stronghold
      WALLET_STRONGHOLD_PASSWORD: some_hopefully_secure_password
      # DATABASE CONNECTION CONFIG
      DB_USER: postgres
      DB_PASSWORD: dlt_booth
      DB_NAME: dlt_booth
      DB_HOST: postgres
      DB_PORT: 5432
      DB_MAX_POOL_SIZE: 16
  postgres:
    container_name: postgres
    hostname: postgres
    image: postgres:latest
    ports:
      - "5432:5432"
    env_file: 
      - ./dlt-booth/env/postgres.env
    volumes: 
      - ./dlt-booth/postgresdata:/var/lib/postgresql/data
      - ./dlt-booth/sql/dbinit.sql:/docker-entrypoint-initdb.d/dbinit.sql 
    restart: always
    healthcheck:
      test: [ "CMD-SHELL", "pg_isready -d $${POSTGRES_DB} -U $${POSTGRES_USER}" ]
      interval: 10s
      timeout: 5s
      retries: 5
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: dlt_booth
      POSTGRES_DB: dlt_booth
    networks:
      - dlt-booth-net
networks:
  dlt-booth-net:
    name: dlt-booth-net
```
## License

[GPL-3.0-or-later](https://spdx.org/licenses/GPL-3.0-or-later.html)