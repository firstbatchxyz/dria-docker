FROM --platform=$BUILDPLATFORM rust:1.71

# https://docs.docker.com/engine/reference/builder/#automatic-platform-args-in-the-global-scope
# Offical rust image supports the following archs: 
#   amd64 (AMD & Intel 64-bit)
#   arm32v7 (ARMv7 32-bit)
#   arm64v8 (ARMv8 64-bit)
#   i386 (Intel 32-bit 8086)
#
# our builds will be for platforms:
#   linux/amd64
#   linux/arm64/v8
#   linux/arm32/v7
#   linux/i386
#
ARG BUILDPLATFORM
ARG TARGETPLATFORM
RUN echo "Build platform:  $BUILDPLATFORM"
RUN echo "Target platform: $TARGETPLATFORM"


RUN apt-get update
RUN apt-get install -y cmake
RUN rustup install nightly-2023-07-25
RUN rustup default nightly-2023-07-25

WORKDIR /usr/src/app

COPY . .

RUN cargo install --path .

EXPOSE 8080

CMD ["dria_hnsw"]
