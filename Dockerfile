FROM rust:1.91.0-slim AS build
WORKDIR /source
COPY . .
RUN apt update && apt install -y wget xz-utils nodejs libfontconfig1-dev libssl-dev openssl musl-tools build-essential
RUN wget -qO- https://get.pnpm.io/install.sh | ENV="$HOME/.shrc" SHELL="$(which sh)" sh -
RUN cd web && rustup target add x86_64-unknown-linux-musl && cargo build --release --locked --target=x86_64-unknown-linux-musl
RUN export PNPM_HOME="/root/.local/share/pnpm" && export PATH="$PNPM_HOME:$PATH" && pnpm i && export VITE_API_PLATFORM=web && pnpm build

FROM alpine:latest as runtime
WORKDIR /app
COPY --from=build /source/web/target/x86_64-unknown-linux-musl/release/web .
COPY --from=build /source/build ./www
RUN chmod +x ./web

ENTRYPOINT ["/app/web"]