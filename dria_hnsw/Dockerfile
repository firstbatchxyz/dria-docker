#TODO: This Dockerfile will result in VERY big images. Needs to be optimized.
FROM rust:1.71

RUN apt-get update
RUN apt-get install -y cmake
RUN rustup install nightly-2023-07-25
RUN rustup default nightly-2023-07-25

WORKDIR /usr/src/app

COPY . .

RUN cargo install --path .

EXPOSE 8080

CMD ["dria_hnsw"]