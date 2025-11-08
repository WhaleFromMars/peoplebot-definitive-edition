# Stage 1: Rust build
FROM rust:1.91-slim-bookworm as builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

RUN cargo build --release

# Stage 2: Actual Runtime
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates wget xz-utils && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Rust binary
COPY --from=builder /app/target/release/peoplebot ./app

# PINNED: yt-dlp binary (static)
ENV YTDLP_TAG=2025.10.22

RUN wget -O /usr/local/bin/yt-dlp \
    https://github.com/yt-dlp/yt-dlp/releases/download/${YTDLP_TAG}/yt-dlp_linux && \
    chmod +x /usr/local/bin/yt-dlp

# PINNED: FFmpeg static binary
ENV FFMPEG_BUILD=ffmpeg-n8.0-32-g44bfe0da61-linux64-gpl-8.0
ENV TAG=autobuild-2025-11-08-13-04

RUN wget -O /tmp/${FFMPEG_BUILD}.tar.xz \
    https://github.com/BtbN/FFmpeg-Builds/releases/download/${TAG}/${FFMPEG_BUILD}.tar.xz && \
    tar -xf /tmp/${FFMPEG_BUILD}.tar.xz -C /tmp && \
    mv /tmp/${FFMPEG_BUILD}/bin/ffmpeg /usr/local/bin/ffmpeg && \
    chmod +x /usr/local/bin/ffmpeg && \
    rm -rf /tmp/${FFMPEG_BUILD}*

ENTRYPOINT ["./app"]
