# mediterraneus-connector-rs

![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)
![Iota](https://img.shields.io/badge/iota-29334C?style=for-the-badge&logo=iota&logoColor=white)

Connector repository. It enable users to interacts with the tangle, with IPFS and with other users. Sample implementation for the Mediterraneus Protocol.

## Initialization
Launch the `docker-compose.yml` 

## Running the Application
```sh
cargo run --release --bin connector-rs -- --rpc-provider "http://127.0.0.1:8545/" --chain-id 31337
```

To generate contract bindings (to be done once or if contract change)
```sh
cargo run --bin abigen -- --contract ERC721Base --abi-source "../mediterraneus-smart-contracts/artifacts/contracts/ERC721Base.sol/ERC721Base.json"
```