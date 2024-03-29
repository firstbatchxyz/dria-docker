# Dria HNSW

Dria HNSW is an API that allows you to permissionlessly search knowledge uploaded to Dria. It works over values downloaded from Arweave to a Redis cache, and reads these values directly from Redis.
It is written in Rust, and several functions respect the machine architecture for efficiency.

## Setup

To run the server, you need to provide a contract ID along with a RocksDB path:

```sh
CONTRACT_ID=<contract-id> ROCKSDB_PATH="/path/to/rocksdb" cargo run
```

Dria HNSW is available as a container:

```sh
docker pull firstbatch/dria-hnsw
```

> [!TIP]
>
> The docker image is cross-compiled & built for multiple architectures, so when you pull the image the most efficient code per your architecture will be downloaded!

To see the available endpoints, refer to [this section](#endpoints) below.

## Endpoints

Dria is an [Actix](https://actix.rs/) server with the following endpoints:

- [`health`](#health)
- [`fetch`](#fetch)
- [`query`](#query)
- [`insert_vector`](#insert_vector)

All endpoints return a response in the following format:

- `success`: a boolean indicating the success of request
- `code`: status code
- `data`: response data

> [!TIP]
>
> If `success` is false, the error message will be written in `data` as a string.

### `HEALTH`

<!-- prettier-ignore -->
```ts
GET /health
```

**A simple healthcheck to see if the server is up.**

Response data:

- A string `"hello world!"`.

### `FETCH`

<!-- prettier-ignore -->
```ts
POST /fetch
```

**Given a list of ids, fetches their corresponding vectors.**

Request body:

- `id`: an array of integers

Response data:

- An array of metadatas, index `i` corresponding to metadata of vector with ID `id[i]`.

### `QUERY`

<!-- prettier-ignore -->
```ts
POST /query
```

**Given a list of ids, fetches their corresponding vectors.**

Request body:

- `vector`: an array of floats corresponding to the embedding vector
- `top_n`: number of results to return
- `query`: (_optional_) the text that belongs to given embedding, yields better results by looking for this text within the results
- `level`: (_optional_) an integer value in range [0, 4] that defines the intensity of search, a larger values takes more time to complete but has higher recall

Response data:

- An array of objects with the following keys:
  - `id`: id of the returned vector
  - `score`: relevance score
  - `metadata`: metadata of the vector

### `INSERT_VECTOR`

<!-- prettier-ignore -->
```ts
POST /insert_vector
```

**Insert a new vector to HNSW.**

Request body:

- `vector`: an array of floats corresponding to the embedding vector
- `metadata`: (_optional_) a JSON object that represent metadata for this vector

Response data:

- A string `"Success"`.

## Testing

We have several tests that you can run with:

```sh
cargo test
```

Some tests expect a RocksDB folder present at `$HOME/.dria/data/WbcY2a-KfDpk7fsgumUtLC2bu4NQcVzNlXWi13fPMlU`, which can easily be downloaded with the [Dria CLI](https://github.com/firstbatchxyz/dria-cli/) if you do not have it:

```sh
dria pull WbcY2a-KfDpk7fsgumUtLC2bu4NQcVzNlXWi13fPMlU
```

The said knowledge is a rather lightweight knowledge that is useful for testing.
