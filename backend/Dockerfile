# Build stage
FROM rust:1-slim-bookworm AS builder

# A C toolchain and cmake for the crates that compile C (mimalloc, aws-lc).
# Package versions follow the pinned Debian release of the base image.
# hadolint ignore=DL3008
RUN apt-get update \
 && apt-get install -y --no-install-recommends build-essential cmake pkg-config \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock build.rs ./
COPY migration/ ./migration/
COPY src/ ./src/

# CARGO_BUILD_JOBS can be lowered at build time on a small or shared server.
ARG CARGO_BUILD_JOBS
RUN cargo build --release --locked

# Runtime stage
FROM debian:bookworm-slim AS runtime

# hadolint ignore=DL3008
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/farm-doctor-api /usr/local/bin/farm-doctor-api

EXPOSE 3000

# Bring the database up to date, then serve.
CMD ["sh", "-c", "farm-doctor-api migrate && exec farm-doctor-api serve"]
