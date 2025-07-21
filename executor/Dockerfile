# Stage 1: Build the application
FROM rust:1.75-slim@sha256:e0b2b5c7e2fb0ad3f7ac44c8ed06d4694931a7201c13d8031a0e0513e9a7e939 AS builder   # pin digest

# Install build dependencies including clang
RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential pkg-config libssl-dev clang

# Set up working directory
WORKDIR /app

# ------------ dependency-cache layer -------------
# Copy Cargo.lock first to leverage Docker's build cache
COPY ./Cargo.lock .
COPY ./executor/Cargo.toml executor/Cargo.toml
COPY ./shared-models/Cargo.toml shared-models/Cargo.toml

# Create dummy source files for dependency build (prevents "file not found" errors)
RUN mkdir -p executor/src shared-models/src && \
    echo 'fn main(){}' > executor/src/main.rs && \
    echo 'pub fn lib(){}' > shared-models/src/lib.rs

# Build only dependencies to cache them (use --workspace as it's a workspace build)
RUN cargo build --release --workspace

# Clean up dummy files
RUN rm executor/src/*.rs shared-models/src/*.rs

# ------------ real source copy and build -------------
# Copy actual application source code
COPY ./executor/src ./executor/src
COPY ./shared-models/src ./shared-models/src

# Build the final executable
RUN cargo build --release --bin executor


# --- Stage 2: Create the final, minimal image ------------------------------
FROM debian:bookworm-slim

# Install only necessary runtime libraries and clean up apt cache
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN groupadd --system app && useradd --system --gid app app
USER app

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/executor .

# Set LIBCLANG_PATH for runtime environment if needed by dynamic libs (e.g., from clang)
ENV LIBCLANG_PATH=/usr/lib/llvm-14/lib

EXPOSE 9184

CMD ["./executor"]
