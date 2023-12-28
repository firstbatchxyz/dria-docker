# Dria HNSW

Dria HNSW is an API that allows you to permissionlessly search knowledge uploaded to Dria. It works over values downloaded from Arweave to a Redis cache, and reads these values directly from Redis.

It is written in Rust, and several functions respect the machine architecture for efficiency. In line with this, the docker image is built for 4 different architectures, as supported by the [offical Rust image](https://hub.docker.com/_/rust/).

## Setup

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
- [`dria/fetch`](#fetch)
- [`dria/query`](#query)
- [`dria/insert`](#insert)

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
POST /dria/fetch
```

**Given a list of ids, fetches their corresponding vectors.**

Request body:

- `id`: an array of integers

Response data:

- An array of metadatas, index `i` corresponding to metadata of vector with ID `id[i]`.

### `QUERY`

<!-- prettier-ignore -->
```ts
POST /dria/query
```

**Given a list of ids, fetches their corresponding vectors.**

Request body:

- `vector`: an array of floats corresponding to the embedding vector
- `top_n`: number of results to return
- `level`: (_optional_) an integer value in range [0, 4] that defines the intensity of search, a larger values takes more time to complete but has higher recall

Response data:

- An array of objects with the following keys:
  - `id`: id of the returned vector
  - `score`: relevance score
  - `metadata`: metadata of the vector

### `INSERT`

<!-- prettier-ignore -->
```ts
POST /dria/insert
```

**Insert a new vector to HNSW.**

Request body:

- `vector`: an array of floats corresponding to the embedding vector
- `metadata`: (_optional_) a JSON object that represent metadata for this vector

Response data:

- A string `"Success"`.
