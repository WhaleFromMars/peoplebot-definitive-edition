# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.91
ARG YT_DLP_VERSION=2025.10.22
ARG DEBIAN_FRONTEND=noninteractive

##
## Builder: compile the Rust binary with cached dependencies.
##
FROM rust:${RUST_VERSION}-slim-bookworm AS chef
ARG DEBIAN_FRONTEND
ENV DEBIAN_FRONTEND=${DEBIAN_FRONTEND}
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    autoconf \
    automake \
    build-essential \
    clang \
    cmake \
    libopus-dev \
    libssl-dev \
    libtool \
    m4 \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo install cargo-chef --locked

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release --locked

##
## Dependency fetcher: download and verify yt-dlp once.
##
FROM debian:bookworm-slim AS yt-dlp-fetcher
ARG DEBIAN_FRONTEND
ENV DEBIAN_FRONTEND=${DEBIAN_FRONTEND}
ARG YT_DLP_VERSION
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*
RUN set -eux; \
    tmpdir="$(mktemp -d)"; \
    cd "${tmpdir}"; \
    curl -L -o yt-dlp "https://github.com/yt-dlp/yt-dlp/releases/download/${YT_DLP_VERSION}/yt-dlp"; \
    curl -L -o SHA256SUMS "https://github.com/yt-dlp/yt-dlp/releases/download/${YT_DLP_VERSION}/SHA256SUMS"; \
    grep "  yt-dlp$" SHA256SUMS > SHA256SUMS.filtered; \
    sha256sum -c SHA256SUMS.filtered; \
    install -m 0755 yt-dlp /usr/local/bin/yt-dlp; \
    rm -rf "${tmpdir}"

##
## Runtime: copy in the slim binary and run as a non-root user.
##
FROM debian:bookworm-slim AS runtime
ARG DEBIAN_FRONTEND
ENV DEBIAN_FRONTEND=${DEBIAN_FRONTEND}
ENV TZ=Etc/UTC
ARG APP_USER=peoplebot
RUN ln -snf "/usr/share/zoneinfo/${TZ}" /etc/localtime && echo "${TZ}" > /etc/timezone
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates \
    ffmpeg \
    libopus0 \
    python3 \
    tzdata \
    && rm -rf /var/lib/apt/lists/*
COPY --from=yt-dlp-fetcher /usr/local/bin/yt-dlp /usr/local/bin/yt-dlp
WORKDIR /app
COPY --from=builder /app/target/release/peoplebot /usr/local/bin/peoplebot
RUN useradd --system --create-home --shell /usr/sbin/nologin "${APP_USER}"
RUN install -d -o "${APP_USER}" -g "${APP_USER}" /app/out /app/tmp
USER ${APP_USER}
ENTRYPOINT ["/usr/local/bin/peoplebot"]
