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
- A Dria contract ID

A Dria contract is the knowledge that is deployed on Arweave; the contract ID can be seen on each knowledge deployed to [Dria](https://dria.co/). For example, consider the Dria knowledge of [The Rust Programming Language](https://dria.co/knowledge/7EZMw0vAAFaKVMNOmu2rFgFCFjRD2C2F0kI_N5Cv6QQ):

- <https://dria.co/knowledge/7EZMw0vAAFaKVMNOmu2rFgFCFjRD2C2F0kI_N5Cv6QQ>

The base64 URL there is our contract ID, and it can also be seen at the top of the page at that link.

### Using Dria CLI

The preferred method of using Dria Docker is via the [Dria CLI](https://github.com/firstbatchxyz/dria-cli/), which is an NPM package.

```sh
npm i -g dria-cli
```

You can see available commands with:

```sh
dria help
```

See the [docs](https://github.com/firstbatchxyz/dria-cli/?tab=readme-ov-file#usage) of Dria CLI for more.

### Using Compose

Download the Docker compose file:

```sh
curl -o compose.yaml -L https://raw.githubusercontent.com/firstbatchxyz/dria-docker/master/compose.yaml
```

You can start a Dria container with the following command, where the contract ID is provided as environment variable.

```sh
CONTRACT=contract-id docker compose up
```

## Usage

When everything is up, you will have access to both Dria and HollowDB on your local network!

- Dria HNSW will be live at `localhost:8080`, see endpoints [here](./dria_hnsw/README.md#endpoints).
- HollowDB API will be live at `localhost:3030`, see endpoints [here](./hollowdb/README.md#endpoints).

These host ports can also be changed within the [compose file](./compose.yaml), if you have them reserved for other applications.

> [!TIP]
>
> You can also connect to a terminal on the Redis container and use `redis-cli` if you would like to examine the keys.

## License

Dria Docker is licensed under [Apache 2.0](./LICENSE).
