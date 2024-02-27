# mediterraneus-connector-rs

![Iota](https://img.shields.io/badge/iota-29334C?style=for-the-badge&logo=iota&logoColor=white)

Connector repository. It enable users to interacts with the tangle, with IPFS and with other users. Sample implementation for the Mediterraneus Protocol.

## Prepare environment

0. Generate contract bindings (to be done once or if contract change)
```sh
# assuming the mediterraneus-smart-contracts folder is located in the same root folder of mediterraneus-connector-rs
cd abigen
cargo run -- --contract ERC721Base --abi-source "../../mediterraneus-smart-contracts/artifacts/contracts/ERC721Base.sol/ERC721Base.json"
```

1. In `connector-rs/env` folder, create a `.env` file starting from `.env.example` and update the values accordingly to your development enviroment. 

## Running the Application

0. Launch [IPFS desktop](https://docs.ipfs.tech/install/ipfs-desktop/)

1. Start up the database by running:
```sh
cd connector-rs
docker compose up -d
```

2. Run the connector service:
```sh
cd connector-rs
cargo run --release -- --rpc-provider "http://127.0.0.1:8545/" --chain-id 31337
```

## License

[GPL-3.0-or-later](https://spdx.org/licenses/GPL-3.0-or-later.html)