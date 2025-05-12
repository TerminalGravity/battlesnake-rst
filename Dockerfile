# Stage 1: Build the application
FROM rust:1.78 as builder

# Set working directory
WORKDIR /app

# Copy manifests and source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Cache dependencies. This layer is rebuilt only when Cargo.toml or Cargo.lock changes.
# Build dummy project to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Build the actual application
# This will use the cached dependencies from the previous step.
COPY src ./src
# Ensure src/main.rs is overwritten if it exists from dummy build
RUN rm -f src/main.rs && touch src/main.rs 
RUN cargo build --release

# Stage 2: Create the final, minimal image
FROM debian:bullseye-slim

# Set environment variables
# Default log level to info, can be overridden
ENV RUST_LOG=info
# The server will bind to this port, Battlesnake hosting expects 8080 usually
ENV PORT=8080 

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/battlesnake-rst .

# Expose the port the app runs on
EXPOSE 8080

# Set the entrypoint command
CMD ["./battlesnake-rst"] 