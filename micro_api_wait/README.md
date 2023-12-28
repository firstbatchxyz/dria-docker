# Micro API `wait-for`

This is a simple shell script container that another container can depend "on completion", such that the script will finish when Micro API finished downloading & refreshing keys.

The containers are expected to launch in the following order:

1. **Redis**: This is the first container to launch.
2. **HollowDB Micro API**: This starts when Redis is live, and immediately begins to download values from Arweave & store them in memory for Dria to access efficiently.
3. **Dria HNSW**: Dria waits for the Micro API's download the complete via [this wait-for script](./wait.sh), and once that is complete; it launched & starts listening at it's port.

The script is available on Docker Hub:

```sh
docker pull firstbatch/dria-hollowdb-wait-for
```

## Wait-For Script

The script makes use of the following cURL command:

```sh
curl -f -d '{"route": "STATE"}' $TARGET
```

If the cache is still loading, Micro API will respond with `503` and cURL will return a non-zero code, causing the shell script to wait for a while and try again later.

The body of the response also contains the percentage of keys loaded, if you are to make the request yourself.
