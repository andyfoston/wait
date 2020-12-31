FROM rust:1.45 as builder
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src/wait
COPY . .
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/wait .
CMD ["./wait"]
