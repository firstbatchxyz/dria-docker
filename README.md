<p align="center">
  <img src="https://raw.githubusercontent.com/firstbatchxyz/dria-js-client/master/logo.svg" alt="logo" width="142">
</p>

<p align="center">
  <h1 align="center">
    Dria Docker
  </h1>
  <p align="center">
    <i>Dria Docker is an all-in-one environment to use <a href="https://dria.co/" target="_blank">Dria</a>, the collective knowledge for AI.</i>
  </p>
</p>

## Setup

To use Dria Docker, you need:

- [Docker](https://www.docker.com/) installed in your machine.
- An [Arweave wallet](https://arweave.app/welcome) in your machine, which you will provide via its path.
- A Dria contract deployed on Arweave to connect to, which you will provide via its transaction ID. The contract ID can be seen on each knowledge deployed to [Dria](https://dria.co/), you can simply use that ID here!

## Usage

Download the Docker compose file:

```sh
curl -o compose.yaml -L https://raw.githubusercontent.com/firstbatchxyz/dria-docker/master/compose.yaml
```

You can start a Dria container with the following command, where the wallet & contract information is provided as environment variables. Note that the wallet path must start with either relative path `./` or absolute path `/`.

```sh
WALLET=./path/to/wallet.json CONTRACT=contract-txid docker compose up
```

When everything is up, you will have access to both Dria and HollowDB on your local network!

- Dria HNSW will be live at `localhost:8080`, see endpoints [here](./dria_hnsw/README.md#endpoints).
- HollowDB Micro API will be live at `localhost:3030`, see endpoints [here](./micro_api/README.md#endpoints).

These host ports can also be changed within the [compose file](./compose.yaml), if you have them reserved for other applications.

> [!TIP]
>
> You can also connect to a terminal on the Redis container and use `redis-cli` if you would like to examine the keys.

## License

Dria Docker is licensed under [Apache 2.0](./LICENSE).
