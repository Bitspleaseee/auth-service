FROM rustlang/rust:nightly

WORKDIR /usr/src/auth-service
COPY . .

RUN cargo install --path . --bin auth-service

CMD ["auth-service", "-v", "-m"]
