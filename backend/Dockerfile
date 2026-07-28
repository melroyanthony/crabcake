# syntax=docker/dockerfile:1
#
# Multi-stage build with cargo-chef so dependency crates stay cached across
# source changes. SQLx is compiled offline from the committed `.sqlx/` cache —
# there is no database during the image build. The runtime image stays free of
# a C toolchain; everything in this crate speaks TLS through ring.

FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ENV SQLX_OFFLINE=true
COPY --from=planner /app/recipe.json recipe.json
# Offline query data has to exist before `cook` if any dependency build were to
# touch it; the final `cargo build` definitely needs it.
COPY .sqlx .sqlx
RUN cargo chef cook --release --recipe-path recipe.json --bin api --bin worker
COPY . .
RUN cargo build --release --bin api --bin worker \
    && strip target/release/api target/release/worker

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home app

COPY --from=builder /app/target/release/api /usr/local/bin/api
COPY --from=builder /app/target/release/worker /usr/local/bin/worker

USER app
EXPOSE 8000
# The same image runs the API and the worker; Compose overrides the command.
CMD ["api"]
