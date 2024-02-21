# HollowDB API

Each Dria knowledge is a smart-contract on Arweave, which serves the knowledge on permaweb via a key-value interface. To download them, we require HollowDB client, which is a Node package. Here we have a Fastify server that acts as an API for HollowDB, which we primarily use to download & unbundle the values for Dria HNSW to use.

## Installation

Install the packages:

```sh
yarn install
```

## Usage

To run the server, you need to provide a contract ID along with a RocksDB path:

```sh
CONTRACT_ID=<contract-id> ROCKSDB_PATH="path/do/rocksdb" yarn start
```

HollowDB PAI is available as a container:

```sh
docker pull firstbatch/dria-hollowdb
```

In both cases, you will need a Redis container running at the URL defined by `REDIS_URL` environment variable.

### Configurations

There are several environment variables to configure the server. You can provide them within the command line, or via `.env` file. An example is given [here](./.env.example).

- `REDIS_URL=<redis-url>` <br> You need a Redis server running before you start the server, the URL to the server can be provided with a `REDIS_URL` environment variable. The connection URL defaults to `redis://default:redispw@localhost:6379`.

- `WARP_LOG_LEVEL=<log-level>` <br> By default Warp will log at `info` level, but you can change it via the `WARP_LOG_LEVEL` environment variable. Options are the known levels of `debug`, `error`, `fatal`, `info`, `none`, `silly`, `trace` and `warn`.

- `USE_BUNDLR=<true/false>` <br> You can treat the values as transaction ids if `USE_BUNDLR` environment variable is set to be `true`. When this is the case, `REFRESH` will actually fetch the uploaded data and download it to Redis.

> [!WARNING]
>
> Uploading to Bundlr via `PUT` or `PUT_MANY` is not yet implemented.

- `USE_HTX=<true/false>` <br> When we have `USE_BUNDLR=true` we treat the stored values as transaction ids; however, HollowDB may have an alternative approach where values are stored as `hash.txid` (due to [this implementation](https://github.com/firstbatchxyz/hollowdb/blob/master/src/contracts/hollowdb-htx.contract.ts)). To comply with this approach, set `USE_HTX=true`.

- `BUNDLR_FBS=<number>` <br> When using Bundlr, downloading values from Arweave cannot be done in a huge `Promise.all`, as it causes timeouts. We instead download values in batches, defaulting to 40 values per batch. To override the batch size, you can provide an integer value to this variable.

## Endpoints

HollowDB API exposes the following endpoints:

- GET [`/state`](#state)
- POST [`/get`](#get)
- POST [`/getRaw`](#getraw)
- POST [`/getMany`](#getmany)
- POST [`/getManyRaw `](#getmanyraw)
- POST [`put`](#put)
- POST [`putMany`](#putmany)
- POST [`update`](#update)
- POST [`remove`](#remove)
- POST [`refresh`](#refresh)
- POST [`clear`](#clear)

### `get`

```ts
interface {
  key: string
}

// response body
interface {
  value: any
}
```

Returns the value at the given key.

### `getRaw`

```ts

// response body
interface {
  value: any
}
```

Returns the value at the given `key`, directly from the cache layer & without involving Warp or Arweave.

### `getMany`

```ts
interface {
  keys: string[]
}

// response body
interface {
  values: any[]
}
```

Returns the values at the given `keys`.

### `getManyRaw`

```ts
interface {
  keys: string[]
}

// response body
interface {
  values: any[]
}
```

Returns the values at the given `keys`, reads directly from the storage.

This has the advantage of not being bound to the interaction size limit, however, the user must check that the data is fresh with their own methods.
Furthermore, you must make a call to `REFRESH` before using this endpoint, and subsequent calls to `REFRESH` will update the data with the new on-chain values.

### `put`

```ts
interface {
  key: string,
  value: any
}
```

Puts `value` at the given `key`. The key must not exist already, or it must have `null` stored at it.

### `putMany`

```ts
interface {
  keys: string[],
  values: any[]
}
```

Updates given `keys` with the provided `values`. No key must exist already in the database.

### `update`

```ts
interface {
  key: string,
  value: any,
  proof?: object
}
```

Updates a `key` with the provided `value` and an optional `proof`.

### `remove`

```ts
interface {
  key: string,
  proof?: object
}
```

Removes the value at `key`, along with an optional `proof`.

### `state`

Syncs & fetches the latest contract state, and returns it.

### `refresh`

Syncs & fetches the latest state and stores the latest sort key for each key in the database. Returns the number of keys refreshed for diagnostic purposes.

### `clear`

```ts
interface {
  keys?: string[]
}
```

Clears the contents for given `keys` with respect to the values written by `REFRESH` endpoint. One might want to refresh some keys again, without flushing the entire database, so that is the purpose of this endpoint. Returns the number of keys cleared for diagnostic purposes.

> [!TIP]
>
> If no `keys` are given to the `CLEAR` endpoint (i.e. `keys = undefined`) then this will clear **all keys**.

## Testing

We have tests that roll a local Arweave and run tests on them with the micro server in the middle.

> [!NOTE]
>
> You need a Redis server running in the background for the tests to work.

To run tests, do:

```sh
yarn test
```
