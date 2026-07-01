ARG TARGET="aarch64-unknown-linux-musl"

FROM --platform=linux/aarch64 rust:1.94-slim-bookworm AS builder
ARG TARGET
WORKDIR /app
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add "${TARGET}"

COPY Cargo.toml Cargo.lock ./
COPY core/Cargo.toml ./core/Cargo.toml
COPY server/Cargo.toml ./server/Cargo.toml
COPY wasm/Cargo.toml ./wasm/Cargo.toml
COPY cli/Cargo.toml ./cli/Cargo.toml

RUN cargo fetch --locked --target ${TARGET}

# Copy the modules we need
COPY core core
COPY server server

RUN CARGO_INCREMENTAL=0 \
		RUSTFLAGS="-C strip=debuginfo -C target-feature=+aes,+sse2,+ssse3" \
		cargo build --release --locked --target ${TARGET} \
		--package passe-server



# -------------------------------------

FROM --platform=linux/aarch64 scratch
ARG TARGET

# cheeky mkdir
WORKDIR /app/server
WORKDIR /app
ENV CARGO_MANIFEST_DIR=/app/server
COPY --from=builder /app/target/${TARGET}/release/passe-server .
COPY wasm/public wasm/public
COPY ui/public ui/public

EXPOSE 8000
CMD ["./passe-server"]
