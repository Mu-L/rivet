# syntax=docker/dockerfile:1.2

FROM rust:1.81.0-slim AS rust

# - Install curl for health checks
# - Install go-migrate for running migrations
# - Install database clients to be able to run `rivet db shell ...` (Redis, Postgres, ClickHouse)
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y \
    protobuf-compiler \
    pkg-config \
    libssl-dev \
    g++ \
    git \
    libpq-dev \
    wget \
    ca-certificates \
    openssl \
    curl \
    redis-tools \
    postgresql-client \
    clickhouse-client && \
    (curl -L https://github.com/golang-migrate/migrate/releases/download/v4.18.1/migrate.linux-amd64.tar.gz | tar xvz) && \
    mv migrate /usr/local/bin/migrate

WORKDIR /usr/rivet

COPY . .

# Build and copy all binaries from target directory into an empty image (it is not
# included in the output because of cache mount)
RUN \
	--mount=type=cache,target=/usr/local/cargo/git \
	--mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=/usr/rivet/target \
	RUSTFLAGS="--cfg tokio_unstable" cargo build --bin rivet && \
	mv target/debug/rivet /usr/bin/rivet && \
	mkdir /etc/rivet
