###############################################
# Stage 1: Build the frontend with Node
###############################################
FROM node:25-bookworm-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm install
COPY frontend/ ./
RUN npm run build

##############################################
# Stage 2: Build the Rust backend (with cargo-chef caching)
##############################################

# Common setup for backend stages
FROM rust:1.91-slim-trixie AS backend-common
WORKDIR /app/backend
RUN cargo install cargo-chef
RUN apt-get update && \
    apt-get install -y libssl-dev pkg-config ca-certificates default-libmysqlclient-dev && \
    rm -rf /var/lib/apt/lists/*

# Planner: compute dependency graph to enable caching
FROM backend-common AS backend-planner
COPY backend/ ./
RUN cargo chef prepare --recipe-path recipe.json

# Cacher: build dependency layers
FROM backend-common AS backend-cacher
COPY --from=backend-planner /app/backend/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Builder: build the actual backend application
FROM backend-common AS backend-builder
COPY backend/ ./
COPY --from=backend-cacher /app/backend/target target
RUN cargo build --release

##############################################
# Stage 3: Runtime image
##############################################
FROM debian:trixie-slim AS runtime
RUN apt-get update && \
    apt-get install -y default-libmysqlclient-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend-builder /app/backend/target/release/backend /app/backend
COPY --from=frontend-builder /app/frontend/dist /app/static

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080

EXPOSE 8080

CMD ["/app/backend"]
