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

# Planner: compute dependency graph to enable caching
FROM rust:1.91-slim-trixie AS backend-planner
WORKDIR /app/backend
RUN cargo install cargo-chef
COPY backend/ ./
RUN cargo chef prepare --recipe-path recipe.json

# Cacher: build dependency layers
FROM rust:1.91-slim-trixie AS backend-cacher
WORKDIR /app/backend
RUN cargo install cargo-chef
COPY --from=backend-planner /app/backend/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Builder: build the actual backend application
FROM rust:1.91-slim-trixie AS backend-builder
WORKDIR /app/backend
COPY backend/ ./
COPY --from=backend-cacher /app/backend/target target
RUN cargo build --release

##############################################
# Stage 3: Runtime image
##############################################
FROM debian:trixie-slim AS runtime
RUN useradd -m -u 10001 appuser
WORKDIR /app
COPY --from=backend-builder /app/backend/target/release/backend /app/backend
COPY --from=frontend-builder /app/frontend/dist /app/static

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8080

EXPOSE 8080
USER appuser

CMD ["/app/backend"]
