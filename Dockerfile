FROM lukemathwalker/cargo-chef:latest-rust-1.93 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin MILFPics

FROM gcr.io/distroless/cc-debian13 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/MILFPics /usr/local/bin/MILFPics

ENTRYPOINT ["/usr/local/bin/MILFPics"]
EXPOSE 3000