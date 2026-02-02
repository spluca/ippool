# Build stage
FROM rust:1.91.1-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create a new empty shell project
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src

# Build the actual application
# Touch main.rs to force rebuild
RUN touch src/main.rs && \
    cargo build --release --locked

# Strip the binary to reduce size
RUN strip /app/target/release/ippool

# Runtime stage
FROM alpine:latest

# Install CA certificates and curl for health check
RUN apk add --no-cache ca-certificates curl

# Create non-root user
RUN addgroup -g 1000 ippool && \
    adduser -D -s /bin/sh -u 1000 -G ippool ippool

# Copy binary from builder
COPY --from=builder /app/target/release/ippool /usr/local/bin/ippool

# Change ownership
RUN chown ippool:ippool /usr/local/bin/ippool

# Switch to non-root user
USER ippool

# Expose port
EXPOSE 8090

# Set default environment variables
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://127.0.0.1:8090/api/v1/health || exit 1

# Run the application
ENTRYPOINT ["/usr/local/bin/ippool"]
CMD []
