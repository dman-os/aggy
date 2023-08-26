# TODO: build as release for prod
# FROM docker.io/library/rust:1.72 AS chef
FROM docker.io/library/rust:1.72-slim AS builder

WORKDIR /srv/app

ENV RUSTFLAGS="--cfg uuid_unstable"

# RUN apt-get update && apt-get install -y \
#   curl \
#   #   pkgconf \
#   && rm -rf /var/lib/apt/lists/*
#
# RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
# RUN cargo binstall cargo-chef -y
#
# # # this is required by the build script of openssl-sys
# # RUN apt-get update && apt-get install -y \
# #   pkgconf \
# #   && rm -rf /var/lib/apt/lists/*
#
# FROM chef AS planner
# COPY . .
# RUN cargo chef prepare --recipe-path recipe.json
#
# # FROM chef AS cacher
# # COPY --from=planner /srv/app/recipe.json recipe.json
# # # RUN cargo chef cook --release --recipe-path recipe.json
#
# FROM chef AS builder
# COPY --from=planner /srv/app/recipe.json recipe.json
# RUN cargo chef cook --recipe-path recipe.json
COPY . .
# Copy over the cached dependencies
# COPY --from=cacher /srv/app/target target
# COPY --from=cacher $CARGO_HOME $CARGO_HOME
ENV SQLX_OFFLINE=true
# RUN cargo build --release --no-default-features 
RUN cargo build -p aggynfrens_api --no-default-features

FROM docker.io/library/rust:1.72-slim AS runtime
WORKDIR /srv/app
# COPY --from=builder /srv/app/target/debug/web /srv/app/target/debug/worker /usr/local/bin/
COPY --from=builder /srv/app/target/debug/aggynfrens_api /usr/local/bin/
CMD aggynfrens_api
