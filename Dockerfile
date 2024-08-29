# ---------------------------------------------------
# 1 - Build Stage
# ---------------------------------------------------

FROM rustlang/rust:nightly-alpine AS build
WORKDIR /usr/src/app
COPY . .
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cd abigen  \ 
    && cargo run -- --contract ServiceBase --abi-source "../smart-contracts/ServiceBase.json" \
    && cd ..
RUN cargo install --path ./connector-rs

# ---------------------------------------------------
# 2 - Deploy Stage
# ---------------------------------------------------

FROM alpine:latest
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
COPY --from=build /usr/local/cargo/bin/connector /usr/local/bin/connector
COPY --from=build /usr/src/app/connector-rs/env/.env ./env/.env
COPY --from=build /usr/src/app/connector-rs/env/postgres.env ./env/postgres.env
EXPOSE 8085
CMD [ "connector" , "--rpc-provider", "https://json-rpc.evm.testnet.shimmer.network", "--chain-id" , "1073" ] 