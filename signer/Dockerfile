# Stage 1: Build the application
FROM rust:1.77-slim as builder

WORKDIR /app

# Create a dummy project to cache dependencies
RUN USER=root cargo new --bin signer
WORKDIR /app/signer

# Copy dependency files
COPY ./Cargo.lock ./Cargo.lock
COPY ./signer/Cargo.toml ./Cargo.toml
COPY ./shared-models/Cargo.toml ./shared-models/Cargo.toml

# Build dependencies
RUN cargo build --release
RUN rm src/*.rs target/release/deps/signer*

# Copy source code and build the actual application
COPY ./signer/src ./src
COPY ./shared-models/src ./shared-models/src
RUN cargo build --release

# Stage 2: Create the final, minimal image
FROM debian:bookworm-slim

# Install only necessary libraries
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN groupadd --system app && useradd --system --gid app app
USER app

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/signer/target/release/signer .

# Create directory for wallet
RUN mkdir -p /app/wallet

CMD ["./signer"]
