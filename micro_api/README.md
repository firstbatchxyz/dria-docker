# HollowDB Micro for Docker

This backend implementation uses [Micro](https://github.com/vercel/micro), which is a very lightweight server for NodeJS by Vercel, tailored towards usage within a container & supports Unix Domain Socket. It supports almost all CRUD operations as exposed by HollowDB; however, it is tailored specifically towards performant GET operations.

In particular, you will have the most performant multiple-GET requests in the following way:

- Call `REFRESH` to populate Redis with the latest key values. The logic here is similar to [`getStorageValues`](https://github.com/warp-contracts/warp/blob/main/src/contract/HandlerBasedContract.ts#L930) function within Warp.
- Call `GET_MANY_RAW` to get values directly from Redis, without using HollowDB or syncing with Warp.
- Call `GET_RAW` to get a single key the same way.

During a `REFRESH`, if the value is already refreshed for its latest `sortKey` then it is not refreshed again, making this whole process a bit more efficient if refresh is called multiple times. It is also a good practice to call `REFRESH` every so often if your contract has often updates.

> [!ALERT]
>
> On first launch, `REFRESH` is called automatically.

To see the available endpoints, refer to [this section](#endpoints) below.

## Container

You can pull a container of HollowDB Micro API from Docker Hub:

```sh
docker pull firstbatch/dria-hollowdb
```

However, you will need a Redis container running at the URL defined by `REDIS_URL` environment variable.

## Setup

First, you need an Arweave wallet. Provide the path to the wallet with the `WALLET_PATH` environment variable.

> [!NOTE]
>
> By convention you can put your wallet under the `config` folder as `wallet.json`, which is where the server will look for if no path is specified:
>
> ```sh
> cat your-wallet.json > ./src/config/wallet.json
> ```

Then, install the packages.

```sh
yarn install
```

### Configurations

There are several environment variables to configure the server. You can provide them within the command line, or via `.env` file. An example is given [here](./.env.example).

- `WALLET_PATH=path/to/wallet.json` <br> HollowDB requires an Arweave wallet, specified by this variable. If none is given, it defaults to `./config/wallet.json`.

- `REDIS_URL=<redis-url>` <br> You need a Redis server running before you start the server, the URL to the server can be provided with a `REDIS_URL` environment variable. The connection URL defaults to `redis://default:redispw@localhost:6379`.

- `WARP_LOG_LEVEL=<log-level>` <br> By default Warp will log at `info` level, but you can change it via the `WARP_LOG_LEVEL` environment variable. Options are the known levels of `debug`, `error`, `fatal`, `info`, `none`, `silly`, `trace` and `warn`.

- `USE_BUNDLR=<true/false>` <br> You can treat the values as transaction ids if `USE_BUNDLR` environment variable is set to be `true`. When this is the case, `REFRESH` will actually fetch the uploaded data and download it to Redis.

> [!WARNING]
>
> Uploading to Bundlr via `PUT` or `PUT_MANY` is not yet implemented.

- `USE_HTX=<true/false>` <br> When we have `USE_BUNDLR=true` we treat the stored values as transaction ids; however, HollowDB may have an alternative approach where values are stored as `hash.txid` (due to [this implementation](https://github.com/firstbatchxyz/hollowdb/blob/master/src/contracts/hollowdb-htx.contract.ts)). To comply with this approach, set `USE_HTX=true`.

- `BUNDLR_FBS=<number>` <br> When using Bundlr, downloading values from Arweave cannot be done in a huge `Promise.all`, as it causes timeouts. We instead download values in batches, defaulting to 40 values per batch. To override the batch size, you can provide an integer value to this variable.

### Changing the Address

The listened address defaults to `0.0.0.0:3000`, and this can be overridden via `-l` option.

```sh
# listen to another TCP endpoint
yarn start -l tcp://hostname:port

# listen to UNIX Domain Socket (more performant)
yarn start -l unix:/path/to/socket.sock
```

## Development

The development server uses `micro-dev` whereas production server uses `micro`. The former will show errors if there are any, whereas the latter will simply shutdown in case of any thrown error without logging it.

> [!ALERT]
>
> `micro-dev` is not compatible with `micro@10.x.x` so you have to download a previous version to use dev environment. In particular, ([v9.4.0](https://www.npmjs.com/package/micro/v/9.4.0)) is required (although v9.4.1 is latest, it is also not MacOS compatible).

You can start a development server with:

```sh
CONTRACT_TXID="contract-txid" yarn dev
```

## Production

Start the production server with:

```sh
CONTRACT_TXID="contract-txid" yarn start
```

## Endpoints

Due to how tiny [Micro](https://github.com/vercel/micro) is, it does not come with routing; so we instead provide the `route` within the POST body. The interface for the POST body, along with the interface of returned data if there is one, is provided below.

- [`GET`](#get)
- [`GET_RAW`](#get_raw)
- [`GET_MANY`](#get_many)
- [`GET_MANY_RAW`](#get_many_raw)
- [`PUT`](#put)
- [`PUT_MANY`](#put_many)
- [`UPDATE`](#update)
- [`REMOVE`](#remove)
- [`STATE`](#state)
- [`REFRESH`](#refresh)
- [`CLEAR`](#clear)

### `GET`

```ts
interface {
  route: "GET",
  data: {
    key: string
  }
}

// response body
interface {
  value: any
}
```

Returns the value at the given key.

> [!TIP]
>
> Alternatively, any HTTP GET request with a non-empty URI is treated as a key query, where the URI represents the key. For example, a GET request at `http://localhost:3000/key-name` returns the value stored at key `key-name`.

### `GET_RAW`

```ts
interface {
  route: "GET_RAW",
  data: {
    key: string
  }
}

// response body
interface {
  value: any
}
```

Returns the value at the given key, directly from the cache layer & without involving Warp or Arweave.

### `GET_MANY`

```ts
interface {
  route: "GET_MANY",
  data: {
    keys: string[]
  }
}

// response body
interface {
  values: any[]
}
```

Returns the values at the given `keys`.

### `GET_MANY_RAW`

```ts
interface {
  route: "GET_MANY_RAW",
  data: {
    keys: string[]
  }
}

// response body
interface {
  values: any[]
}
```

Returns the values at the given `keys`, reads directly from the storage.

This has the advantage of not being bound to the interaction size limit, however, the user must check that the data is fresh with their own methods.
Furthermore, you must make a call to `REFRESH` before using this endpoint, and subsequent calls to `REFRESH` will update the data with the new on-chain values.

### `PUT`

```ts
interface {
  route: "PUT",
  data: {
    key: string,
    value: any
  }
}
```

Puts `value` at the given `key`. The key must not exist already, or it must have `null` stored at it.

### `PUT_MANY`

```ts
interface {
  route: "PUT_MANY",
  data: {
    keys: string[],
    values: any[]
  }
}
```

Updates given `keys` with the provided `values`. No key must exist already in the database.

### `UPDATE`

```ts
interface {
  route: "UPDATE",
  data: {
    key: string,
    value: any,
    proof?: object
  }
}
```

Updates a `key` with the provided `value` and an optional `proof`.

### `REMOVE`

```ts
interface {
  route: "REMOVE",
  data: {
    key: string,
    proof?: object
  }
}
```

Removes the value at `key`, along with an optional `proof`.

### `STATE`

```ts
interface {
  route: "STATE"
}
```

Syncs & fetches the latest contract state, and returns it.

### `REFRESH`

```ts
interface {
  route: "REFRESH"
}
```

Syncs & fetches the latest state and stores the latest sort key for each key in the database. Returns the number of keys refreshed for diagnostic purposes.

### `CLEAR`

```ts
interface {
  route: "CLEAR"
  data: {
    keys?: string[]
  }
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
