FROM rust:1.88 as builder
WORKDIR /app

RUN cargo install diesel_cli --no-default-features --features postgres

# first cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN mkdir -p api/src && echo "fn dummy() {}" > api/src/lib.rs
COPY api/Cargo.toml ./api/Cargo.toml
RUN cargo build
RUN rm src/main.rs
RUN rm api/src/lib.rs

# build whole package
COPY src ./src
COPY api/src ./api/src
RUN cargo build --release


FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY start.sh diesel.toml ./
COPY migrations ./migrations

# make sure start script is executable
RUN chmod +x /app/start.sh

# after builder job is done, copy binary
COPY --from=builder /app/target/release/bullseye /app/bullseye

# copy diesel bin as well
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/diesel

# execute script with non-root user
RUN useradd -m -s /bin/bash appuser
RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 3000

# ENTRYPOINT ["./bullseye"]
CMD ["./start.sh"]


