# Dria Dockerized

To get started:

```sh
WALLET=path/to/wallet.json CONTRACT=contract-txid docker compose up
```

Or, you can put these into an `.env` and do:

```sh
source .env && docker compose up
```

## Containers

There are three containers here:

1. Redis
2. HollowDB Micro API
3. Dria HNSW

First, Redis container will be setup. Then, HollowDB Micro API will fetch the contract state and update its keys with actual values for Dria HNSW to use. This process takes some time, so we have a healthcheck during this phase:

```sh
curl -f -d '{"route": "STATE"}' http://localhost:3000/
```

If the cache is still loading, this will respond `503` and will exit with a non-zero code, indicating to Docker that our container is not yet ready. Once that is all done, Dria HNSW will be launched.
