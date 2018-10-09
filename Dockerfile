FROM rustlang/rust:nightly

WORKDIR /usr/src/auth-service
COPY . .

RUN cargo install --path .

CMD ["auth-service"]
