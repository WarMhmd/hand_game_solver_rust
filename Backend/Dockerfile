# Stage 1: Builder (with dependency caching)
FROM rust:1.92-slim-bookworm as builder

WORKDIR /app

# 1. Create a dummy project to build dependencies
RUN USER=root cargo new --bin hand_game_solver_rust
WORKDIR /app/hand_game_solver_rust

# 2. Copy only dependency manifests first (for better caching)
# This layer will be cached unless Cargo.toml or Cargo.lock changes
COPY Cargo.toml Cargo.lock ./

# 3. Build dependencies only (this layer will be cached if Cargo files don't change)
# The target directory will persist in the layer, so dependencies won't rebuild
RUN cargo build --release && \
    rm src/*.rs

# 4. Copy actual source code
# This layer only invalidates if source files change
COPY src ./src

# 5. Build the actual application (only rebuilds if source changed)
# Dependencies are already compiled, so this is fast
# Cargo will automatically detect source file changes via COPY
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install OpenSSL, CA certificates, and curl for health checks
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd -ms /bin/bash appuser
USER appuser
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/hand_game_solver_rust/target/release/hand_game_solver_rust .

# Expose the port
EXPOSE 3000

# Run the binary
CMD ["./hand_game_solver_rust"]
