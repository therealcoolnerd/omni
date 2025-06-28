# Multi-stage build for optimal image size
FROM rust:1.70-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies - this layer will be cached unless Cargo.toml changes
RUN cargo build --release && rm -rf src target/release/deps/omni*

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    curl \
    wget \
    gnupg \
    apt-transport-https \
    software-properties-common \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 omni && \
    mkdir -p /home/omni/.config /home/omni/.cache /home/omni/.local/share && \
    chown -R omni:omni /home/omni

# Copy the binary from builder stage
COPY --from=builder /app/target/release/omni /usr/local/bin/omni

# Set proper permissions
RUN chmod +x /usr/local/bin/omni

# Switch to non-root user
USER omni
WORKDIR /home/omni

# Create volume for persistent data
VOLUME ["/home/omni/.config", "/home/omni/.cache", "/home/omni/.local/share"]

# Expose any ports if needed (none for CLI tool)
# EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD omni --version || exit 1

# Set entrypoint
ENTRYPOINT ["omni"]
CMD ["--help"]

# Labels for better metadata
LABEL org.opencontainers.image.title="Omni Universal Package Manager"
LABEL org.opencontainers.image.description="Universal Cross-Platform Package Manager for Linux, Windows, and macOS"
LABEL org.opencontainers.image.vendor="Omni Team"
LABEL org.opencontainers.image.version="0.2.0"
LABEL org.opencontainers.image.source="https://github.com/therealcoolnerd/omni"
LABEL org.opencontainers.image.documentation="https://github.com/therealcoolnerd/omni/blob/main/README.md"