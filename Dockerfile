FROM node:24-alpine AS frontend

WORKDIR /app/web

COPY web/package*.json ./

RUN npm i

COPY web/ .

RUN npm run build


FROM rust:1.98-bookworm AS builder

WORKDIR /app

COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src/ ./src/
COPY ./.sqlx ./.sqlx

ENV SQLX_OFFLINE=true

RUN cargo build --release --locked


FROM rust:1.98.0-slim-bookworm AS runtime

WORKDIR /app

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder \
  /app/target/release/dashboard \
  /usr/local/bin/dashboard

COPY --from=frontend \
  /app/web/dist \
  /app/web/dist

ENV RUST_LOG=info

EXPOSE 8080

CMD ["dashboard"]
