# ---------------------------------------------------
# 1 - Build Stage
# ---------------------------------------------------

FROM rust:1.81.0-alpine AS connector-build
WORKDIR /app
COPY ./connector-rs .
COPY ./smart-contracts /smart-contracts
RUN  apk add --no-cache make musl-dev clang llvm gcc libc-dev clang-dev binutils g++ linux-headers libstdc++ libgcc libressl-dev
# Since the cache is unmounted, I need to move the generated executable in another place
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release && \
    cp target/release/connector /app/connector

# ---------------------------------------------------
# 2 - Deploy Stage
# ---------------------------------------------------
    
 FROM alpine:latest
 COPY --from=connector-build /app/connector /usr/local/bin/connector
 EXPOSE 8085
 CMD ["connector"] 