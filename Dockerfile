# Build image
FROM rustlang/rust:nightly as build

WORKDIR /usr/src/auth-service

# Copy manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Copy source tree
COPY ./src ./src

# Build for release
RUN cargo build --release

# Final image
FROM debian:stable-slim

# Install mariadb client
RUN apt-get update
RUN apt-get -y install libmariadbclient-dev

# Copy the binaries
WORKDIR /usr/src/
COPY --from=build /usr/src/auth-service/target/release/auth-service .
COPY --from=build /usr/src/auth-service/target/release/inspector .

# Set the startup command to run the binary
CMD ["./auth-service", "-m", "-v", "-v"]