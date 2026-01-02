FROM --platform=$BUILDPLATFORM rust:1.91.0-slim AS build
WORKDIR /source
COPY . .
ARG VITE_APP_VERSION
ARG TARGETPLATFORM
ARG BUILDPLATFORM
RUN apt update && apt install -y wget xz-utils nodejs libfontconfig1-dev libssl-dev openssl build-essential cmake gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
RUN wget -qO- https://get.pnpm.io/install.sh | ENV="$HOME/.shrc" SHELL="$(which sh)" sh -
RUN chmod +x ./build-web.sh && ./build-web.sh
RUN export PNPM_HOME="/root/.local/share/pnpm" && export PATH="$PNPM_HOME:$PATH" && pnpm i && export VITE_API_PLATFORM=web && pnpm build

FROM debian:stable-slim as runtime
WORKDIR /app
COPY --from=build /source/web/target/release/web .
COPY --from=build /source/build ./www
RUN chmod +x ./web

ENTRYPOINT ["/app/web"]