# Stage 1: Chef (Prepare dependencies)
# We use a trick to cache dependencies.
FROM rust:1.83-slim-bookworm as builder

WORKDIR /app

# 1. Create a dummy project shell to build dependencies
RUN USER=root cargo new --bin hand_game_solver_rust
WORKDIR /app/hand_game_solver_rust

# 2. Copy manifests only
COPY Cargo.toml Cargo.lock ./

# 3. Build only the dependencies (this layer will be cached!)
RUN cargo build --release
RUN rm src/*.rs

# 4. Now copy your actual source code
COPY src ./src

# 5. Build for release (only compiles your code, linking against cached deps)
# We touch the main file to force a re-compile of the binary itself
RUN touch src/main.rs
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install OpenSSL and CA certificates
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
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
