FROM rust:1.76.0 as builder
WORKDIR /usr/src/rust_docker
RUN apt install libpq-dev
COPY . . 
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/rust_docker /usr/local/bin/rust_docker
EXPOSE 8080
CMD ["rust_docker"]