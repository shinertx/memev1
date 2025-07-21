FROM rust:1.77-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/meta_allocator .
CMD ["./meta_allocator"]
