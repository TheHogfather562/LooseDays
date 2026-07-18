# syntax=docker/dockerfile:1
FROM rust:1-slim-bookworm AS build
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
# Cache the cargo registry/git and the target dir across builds. On the
# self-hosted deploy runner these persist between deploys, so unchanged
# dependencies are neither re-downloaded nor recompiled — only changed
# source recompiles incrementally. target/ is a cache mount (not part of
# the image), so the binary must be copied out within the same RUN.
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release && \
    cp target/release/loosedays-backend /usr/local/bin/loosedays-backend

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/local/bin/loosedays-backend /usr/local/bin/loosedays-backend
EXPOSE 8080
CMD ["loosedays-backend"]
