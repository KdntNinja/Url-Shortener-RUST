# Use the official Rust image as a base
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs file to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build the dependencies
RUN cargo build --release

# Remove the dummy main.rs file
RUN rm -f src/main.rs

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Use a minimal base image for the final stage
FROM debian:buster-slim

# Copy the built binary from the build stage
COPY --from=0 /app/target/release/url_shortener /usr/local/bin/url_shortener

# Expose the port the app runs on
EXPOSE 8080

# Run the application
CMD ["url_shortener"]