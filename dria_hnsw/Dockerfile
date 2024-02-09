FROM --platform=$BUILDPLATFORM rust:1.71 as builder

# https://docs.docker.com/engine/reference/builder/#automatic-platform-args-in-the-global-scope
#
# offical rust image supports the following archs: 
#   amd64 (AMD & Intel 64-bit)
#   arm32/v7 (ARMv7 32-bit)
#   arm64/v8 (ARMv8 64-bit)
#   i386 (Intel 32-bit 8086)
#
# our builds will be for platforms:
#   linux/amd64
#   linux/arm64/v8
#   linux/arm32/v7
#   linux/i386
#
# however, for small image size we use distroless, which allow
#   linux/amd64
#   linux/arm64
#   linux/arm
#
# To build an image & push them to Docker hub for this Dockerfile:
#
# docker buildx build --platform=linux/amd64,linux/arm64,linux/arm . -t firstbatch/dria-hnsw:latest --builder=dria-builder --push   

ARG BUILDPLATFORM
ARG TARGETPLATFORM
RUN echo "Build platform:  $BUILDPLATFORM"
RUN echo "Target platform: $TARGETPLATFORM"

# install Cmake
RUN apt-get update
RUN apt-get install -y cmake

# libclang needed by rocksdb
RUN apt-get install -y clang

# use nightly Rust
RUN rustup install nightly-2023-07-25
RUN rustup default nightly-2023-07-25

# build release binary
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# copy release binary to distroless
FROM --platform=$BUILDPLATFORM gcr.io/distroless/cc
COPY --from=builder /usr/src/app/target/release/dria_hnsw /

EXPOSE 8080

CMD ["./dria_hnsw"]
