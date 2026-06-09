FROM rust:1.84-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY relay/ ./relay/
COPY bridge/ ./bridge/

RUN cargo build --release -p agent-relay -p agent-bridge

FROM debian:bookworm-slim AS relay
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/agent-relay /usr/local/bin/
EXPOSE 8080
CMD ["agent-relay"]

FROM debian:bookworm-slim AS bridge
RUN apt-get update && apt-get install -y tmux ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/agent-bridge /usr/local/bin/
CMD ["agent-bridge", "run"]
