# Build stage using cargo-chef for caching dependencies
FROM rust:1 AS chef

RUN rustup default stable
RUN cargo install cargo-chef
WORKDIR /app

# Plan dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies first to leverage Docker layer caching
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy full source and build the project
COPY . .
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --root /.cargo -y --force
ENV PATH="/.cargo/bin:$PATH"

RUN dx bundle --package web

# Runtime stage
FROM debian:bookworm-slim AS runtime

RUN apt update && apt install -y ca-certificates libssl3 openssl && rm -rf /var/lib/apt/lists/*

VOLUME /usr/local/app/data

COPY --from=builder /app/target/dx/web/release/web /usr/local/app/web

# Set the environment variables
ENV PORT=8080
ENV IP=0.0.0.0


# Expose port
EXPOSE 8080
WORKDIR /usr/local/app

# Start the web app
ENTRYPOINT [ "/usr/local/app/web/server" ]
