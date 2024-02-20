import { SetSDK } from "hollowdb";
import { Redis } from "ioredis";
import { makeServer } from "./server";
import configurations from "./configurations";
import { createCaches, makeWarp } from "./clients/hollowdb";

async function main() {
  const contractId = process.env.CONTRACT_ID;
  if (!contractId) {
    throw new Error("Please provide CONTRACT_ID environment variable.");
  }
  if (Buffer.from(contractId, "base64").toString("hex").length !== 64) {
    throw new Error("Invalid CONTRACT_ID.");
  }

  // ping redis to make sure connection is there before moving on
  const redisClient = new Redis(configurations.REDIS_URL);
  await redisClient.ping();

  // create Redis caches & use them for Warp
  const caches = createCaches(contractId, redisClient);
  const warp = makeWarp(caches);

  // create a random wallet, which is ok since we only make read operations
  // TODO: or, we can use a dummy throw-away wallet every time?
  const wallet = await warp.generateWallet();

  const hollowdb = new SetSDK(wallet.jwk, contractId, warp);
  const server = await makeServer(hollowdb, configurations.ROCKSDB_PATH);
  const addr = await server.listen({
    port: configurations.PORT,
    // host is set to listen on all interfaces to allow Docker internal network to work
    // see: https://fastify.dev/docs/latest/Reference/Server/#listentextresolver
    host: "::",
    listenTextResolver: (address) => {
      return `HollowDB is listening at ${address}`;
    },
  });
  server.log.info(`Listening at: ${addr}`);
}

main();
