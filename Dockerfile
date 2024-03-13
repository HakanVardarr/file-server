ARG RUST_VERSION=1.74.0
ARG APP_NAME=file

FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME
WORKDIR /app

RUN apt-get update && apt-get install -y libsqlite3-dev


RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/server
EOF


FROM debian:bullseye-slim AS final

RUN apt-get update && apt-get install -y libsqlite3-0

COPY database.db /data/database.db

COPY --from=build /bin/server /bin/

EXPOSE 8080

CMD ["/bin/server"]
