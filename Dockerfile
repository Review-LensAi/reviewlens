# --- Stage 1: Build the binary ---
# We use a specific version of Rust to ensure builds are reproducible.
FROM rust:1.82 as builder

WORKDIR /app

# Copy manifests to cache dependencies. This is a key part of making Docker builds fast.
COPY Cargo.toml Cargo.lock ./
COPY crates/engine/Cargo.toml ./crates/engine/
COPY crates/cli/Cargo.toml ./crates/cli/

# Create dummy source files. This allows us to build and cache just the dependencies.
RUN mkdir -p crates/engine/src && echo "pub fn dummy() {}" > crates/engine/src/lib.rs
RUN mkdir -p crates/cli/src && echo "fn main() {}" > crates/cli/src/main.rs

# Build only the dependencies to cache them as a separate Docker layer.
RUN cargo build --release --locked

# Now, copy the actual source code.
COPY crates/engine/src/ ./crates/engine/src/
COPY crates/cli/src/ ./crates/cli/src/

# Build the application. This will be much faster due to the cached dependencies.
RUN cargo build --release --locked --bin reviewlens

# --- Stage 2: Create the final, small image ---
# We use a minimal Debian image for a small and secure final container.
FROM debian:bullseye-slim

# Copy the compiled binary from the builder stage.
COPY --from=builder /app/target/release/reviewlens /usr/local/bin/reviewlens

# Set a default working directory for when the container runs.
WORKDIR /work

# Set the binary as the entrypoint for the container.
ENTRYPOINT ["reviewlens"]

# Set a default command to run when the container starts.
CMD ["--help"]
