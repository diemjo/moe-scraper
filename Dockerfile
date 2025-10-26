ARG RUST_VERSION=1.90.0
ARG APP_NAME=moe-scraper
FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

RUN apt-get update -y && apt-get install -y libssl-dev libsqlite3-dev libfindbin-libs-perl make

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=bind,source=resources/migrations,target=resources/migrations \
    --mount=type=bind,source=templates,target=templates \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/server
EOF

FROM ubuntu:25.10 AS final
RUN apt-get update -y && apt-get install -y libssl-dev libsqlite3-dev openssl ca-certificates

ARG UID=1000
ARG GID=1001
USER $UID:$GID

COPY --from=build /bin/server /bin/
CMD ["/bin/server"]