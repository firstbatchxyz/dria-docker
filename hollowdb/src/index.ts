import { existsSync, readFileSync } from "fs";
import { SetSDK } from "hollowdb";
import { Redis } from "ioredis";
import { makeServer } from "./server";
import configurations from "./configurations";
import { createCaches, makeWarp } from "./clients/hollowdb";
import type { JWKInterface } from "warp-contracts";

const contractTxId = process.env.CONTRACT_TXID;
if (!contractTxId) {
  throw new Error("Please provide CONTRACT_TXID environment variable.");
}
if (Buffer.from(contractTxId, "base64").toString("hex").length !== 64) {
  throw new Error("Invalid CONTRACT_TXID.");
}
const redisClient = new Redis(configurations.REDIS_URL, {
  lazyConnect: false, // explicitly connect
});

const caches = createCaches(contractTxId, redisClient);
if (!existsSync(configurations.WALLET_PATH)) {
  throw new Error("No wallet found at: " + configurations.WALLET_PATH);
}
const wallet = JSON.parse(readFileSync(configurations.WALLET_PATH, "utf-8")) as JWKInterface;
const warp = makeWarp(caches);

const hollowdb = new SetSDK(wallet, contractTxId, warp);

makeServer(hollowdb, configurations.ROCKSDB_PATH).then(async (server) => {
  const addr = await server.listen({ port: configurations.PORT, host: "http://localhost" });
  console.log(`Listening at: ${addr}`);
});
