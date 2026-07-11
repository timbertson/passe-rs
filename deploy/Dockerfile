ARG TARGET="x86_64-unknown-linux-musl"
ARG BUILD_PLATFORM="linux/amd64"

FROM --platform=${BUILD_PLATFORM} rust:1.94-slim-bookworm AS builder
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

FROM --platform=${BUILD_PLATFORM} scratch
ARG TARGET

# cheeky mkdir
WORKDIR /app/server
WORKDIR /app
VOLUME /var/passe
ENV CARGO_MANIFEST_DIR=/app/server
ENV PASSE_SERVER_ROOT=/var/passe
COPY --from=builder /app/target/${TARGET}/release/passe-server .
COPY wasm/public wasm/public
COPY ui/public ui/public

EXPOSE 8080
CMD ["./passe-server"]
