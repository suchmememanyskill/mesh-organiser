FROM rust:1.91.0-slim AS build
WORKDIR /source
COPY . .
RUN apt update && apt install -y wget xz-utils nodejs libfontconfig1-dev libssl-dev openssl musl-tools build-essential
RUN wget -qO- https://get.pnpm.io/install.sh | ENV="$HOME/.shrc" SHELL="$(which sh)" sh -
RUN cd web && rustup target add x86_64-unknown-linux-musl && cargo build --release --locked --target=x86_64-unknown-linux-musl
RUN wget -O mesh-thumbnail https://github.com/suchmememanyskill/mesh-thumbnail/releases/download/v1.7/mesh-thumbnail-x86_64-unknown-linux-gnu
RUN export PNPM_HOME="/root/.local/share/pnpm" && export PATH="$PNPM_HOME:$PATH" && pnpm i && export VITE_API_PLATFORM=web && pnpm build

FROM debian:bookworm-slim as runtime
WORKDIR /app
COPY --from=build /source/web/target/x86_64-unknown-linux-musl/release/web .
COPY --from=build /source/mesh-thumbnail .
COPY --from=build /source/build ./www
COPY start-web.sh .
RUN apt update && apt install -y libfreetype6 libfontconfig xvfb libxcursor-dev libxi-dev && apt-get clean && rm -rf /var/lib/apt/lists/*
RUN chmod +x ./web ./mesh-thumbnail ./start-web.sh

ENV MESH_THUMBNAIL_EXECUTABLE_PATH=/app/mesh-thumbnail

ENTRYPOINT ["/app/start-web.sh"]