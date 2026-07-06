# syntax=docker/dockerfile:1

FROM rust:alpine AS chef
WORKDIR /app
RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef --locked
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:alpine AS builder
ARG TARGETARCH
WORKDIR /app
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN apk add --no-cache binutils musl-dev
COPY --from=chef /usr/local/cargo/bin/cargo-chef /usr/local/cargo/bin/cargo-chef
COPY --from=chef /app/recipe.json recipe.json
RUN set -eux; \
    case "${TARGETARCH:-$(uname -m)}" in \
        amd64|x86_64) RUST_TARGET="x86_64-unknown-linux-musl" ;; \
        arm64|aarch64) RUST_TARGET="aarch64-unknown-linux-musl" ;; \
        *) echo "unsupported TARGETARCH=${TARGETARCH:-$(uname -m)}" >&2; exit 1 ;; \
    esac; \
    rustup target add "$RUST_TARGET"; \
    cargo chef cook --release --locked --target "$RUST_TARGET" --recipe-path recipe.json
COPY . .
RUN set -eux; \
    case "${TARGETARCH:-$(uname -m)}" in \
        amd64|x86_64) RUST_TARGET="x86_64-unknown-linux-musl" ;; \
        arm64|aarch64) RUST_TARGET="aarch64-unknown-linux-musl" ;; \
        *) echo "unsupported TARGETARCH=${TARGETARCH:-$(uname -m)}" >&2; exit 1 ;; \
    esac; \
    cargo build --release --locked --target "$RUST_TARGET" -p amar; \
    strip "target/${RUST_TARGET}/release/amar"; \
    cp "target/${RUST_TARGET}/release/amar" /tmp/amar

FROM scratch
LABEL org.opencontainers.image.title="amar" \
      org.opencontainers.image.description="Offline astronomical tide API with versioned NOAA and REFMAR packs" \
      org.opencontainers.image.source="https://github.com/Destynova2/amar" \
      org.opencontainers.image.licenses="Apache-2.0"
COPY --from=builder /tmp/amar /amar
COPY --from=builder /app/data/packs/noaa_m0.json /packs/noaa_m0.json
COPY --from=builder /app/data/packs/amar-data-brest-experimental.json /packs/amar-data-brest-experimental.json
COPY --from=builder /app/data/packs/amar-data-france-experimental.json /packs/amar-data-france-experimental.json
USER 65534:65534
EXPOSE 3000
# No HEALTHCHECK: scratch has no shell; HTTP probes belong to the orchestrator.
ENTRYPOINT ["/amar"]
CMD ["serve", "--addr", "0.0.0.0:3000", "--pack", "/packs/noaa_m0.json", "--pack", "/packs/amar-data-brest-experimental.json", "--pack", "/packs/amar-data-france-experimental.json"]
