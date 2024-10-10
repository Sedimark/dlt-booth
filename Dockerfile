# ---------------------------------------------------
# 1 - Build Stage
# ---------------------------------------------------

FROM rust:1.81.0-alpine AS build
WORKDIR /app
COPY . .
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN --mount=type=cache,target=/app/connector-rs/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cd connector-rs && cargo build --release && cp target/release/connector /app/connector

# ---------------------------------------------------
# 2 - Deploy Stage
# ---------------------------------------------------

 FROM alpine:latest
RUN  apk add --no-cache gcc libressl-dev
COPY --from=build /app/connector /usr/local/bin/connector
COPY --from=build /app/connector-rs/env/.env ./env/.env
COPY --from=build /app/connector-rs/env/postgres.env ./env/postgres.env
 EXPOSE 8085
 CMD [ "connector" , "--rpc-provider", "https://json-rpc.evm.stardust.linksfoundation.com/", "--chain-id" , "1074" ] 