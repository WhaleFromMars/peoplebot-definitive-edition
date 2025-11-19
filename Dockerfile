# PINNED Versions:
# yt-dlp binary (static)
ARG YTDLP_TAG=2025.11.12
# FFmpeg static binary
ARG FFMPEG_BUILD=ffmpeg-n8.0-32-g44bfe0da61-linux64-gpl-8.0
ARG FFMPEG_TAG=autobuild-2025-11-08-13-04
# Deno for yt_dlps youtube functionality
ARG DENO=v2.5.6

# ========= Stage 1: Chef (Rust + cargo-chef prebuilt) =========
FROM lukemathwalker/cargo-chef:latest-rust-1-slim-bookworm AS chef

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ========= Stage 2: Planner (compute recipe.json) =========
FROM chef AS planner

WORKDIR /app
COPY . .

RUN cargo chef prepare --recipe-path recipe.json

# ========= Stage 3: Builder (cache deps + build app) =========
FROM chef AS builder

WORKDIR /app

# 1) Restore the recipe and build only dependencies (cached layer)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# 2) Copy full source and build binary
COPY . .

RUN cargo build --release

# ========= Stage 4: Runtime =========
FROM debian:bookworm-slim

ARG YTDLP_TAG
ARG FFMPEG_BUILD
ARG FFMPEG_TAG
ARG DENO

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates wget xz-utils unzip && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Rust binary
COPY --from=builder /app/target/release/peoplebot ./app

# yt_dlp install
RUN wget -O /usr/local/bin/yt-dlp \
    https://github.com/yt-dlp/yt-dlp/releases/download/${YTDLP_TAG}/yt-dlp_linux && \
    chmod +x /usr/local/bin/yt-dlp

RUN yt-dlp --version # confirm its available

# FFmpeg install
RUN wget -O /tmp/${FFMPEG_BUILD}.tar.xz \
    https://github.com/BtbN/FFmpeg-Builds/releases/download/${FFMPEG_TAG}/${FFMPEG_BUILD}.tar.xz && \
    tar -xf /tmp/${FFMPEG_BUILD}.tar.xz -C /tmp && \
    mv /tmp/${FFMPEG_BUILD}/bin/ffmpeg /usr/local/bin/ffmpeg && \
    chmod +x /usr/local/bin/ffmpeg && \
    rm -rf /tmp/${FFMPEG_BUILD}*

RUN ffmpeg -version # confirm its available

# Deno Install
RUN curl -fsSL "https://github.com/denoland/deno/releases/download/${DENO}/deno-x86_64-unknown-linux-gnu.zip" -o deno.zip \
    && unzip deno.zip \
    && mv deno /usr/local/bin/deno \
    && chmod +x /usr/local/bin/deno \
    && rm deno.zip \
    && rm -rf /var/lib/apt/lists/*

RUN deno --version # confirm its available

ENTRYPOINT ["./app"]
