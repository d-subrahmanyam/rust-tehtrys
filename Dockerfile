# syntax=docker/dockerfile:1
# Reproducible Linux build + test. `docker build -t tehtrys .` / `kiri build -t tehtrys .`
# Cross-platform note: containers test LINUX; macOS/Windows need native runners or `cross`. See README.
FROM rust:slim

WORKDIR /app

# SDL2 system lib (link). not sdl2_mixer/sdl2_ttf.
RUN apt-get update \
    && apt-get install -y --no-install-recommends libsdl2-dev pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# drop machine-specific linuxbrew env; use container SDL2.
ENV LIBRARY_PATH=/usr/lib/x86_64-linux-gnu \
    LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu

COPY . .
# remove committed host-specific cargo env inside the image.
RUN rm -f .cargo/config.toml

RUN cargo build --locked
RUN cargo test --locked

CMD ["cargo", "run", "--release"]
