# --- 1. frontend build ---
FROM node:20-bookworm AS web
WORKDIR /build/web
COPY live_server/web/package*.json ./
RUN npm ci
COPY live_server/web ./
RUN npm run build

# --- 2. rust build ---
FROM rust:1-bookworm AS builder
WORKDIR /build
COPY . .
RUN cargo build --release -p live_server

# --- 3. runtime ---
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/target/release/live_server /app/live_server
COPY --from=web     /build/static                   /app/static
ENV STATIC_DIR=/app/static
ENV HOST=0.0.0.0
ENV PORT=3030
EXPOSE 3030
CMD ["/app/live_server"]
