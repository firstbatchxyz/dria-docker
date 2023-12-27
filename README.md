<p align="center">
  <h1 align="center">
    Dria Docker
  </h1>
  <p align="center">
    <i>Dria Docker is an all-in-one environment to use Dria, the collective knowledge for AI.</i>
  </p>
</p>

## Setup

To use Dria Docker, you need:

- [Docker](https://www.docker.com/) installed in your machine.
- An Arweave wallet in your machine, which you will provide via its path.
- A contract deployed on Arweave to connect to, which you will provide via its transaction ID.

## Usage

You can start a Dria container with the following command, where the wallet & contract information is provided as environment variables:

```sh
WALLET=path/to/wallet.json CONTRACT=contract-txid docker compose up
```

When everything is up, you will have access to both Dria and HollowDB:

- Dria HNSW will be live at `localhost:8080`
- HollowDB Micro API will be live at `localhost:3030`

These ports can also be changed within the [compose file](./compose.yaml), if you have them reserved for other applications.
