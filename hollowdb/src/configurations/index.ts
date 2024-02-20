import type { LogLevel } from "fastify";
import type { LoggerFactory } from "warp-contracts";

export default {
  /** Port that is listened by HollowDB. */
  PORT: parseInt(process.env.PORT ?? "3030"),
  /** Redis URL to connect to. Defaults to `redis://default:redispw@localhost:6379`. */
  REDIS_URL: process.env.REDIS_URL ?? "redis://default:redispw@localhost:6379",
  /** Path to Rocksdb storage. */
  ROCKSDB_PATH: process.env.ROCKSDB_PATH ?? "./data/values",
  /** Treat values as Bundlr txIds. */
  USE_BUNDLR: process.env.USE_BUNDLR ? process.env.USE_BUNDLR === "true" : process.env.NODE_ENV !== "test",
  /** Use the optimized [`hash.txid`](https://github.com/firstbatchxyz/hollowdb/blob/master/src/contracts/hollowdb-htx.contract.ts) layout for values. */
  USE_HTX: process.env.USE_HTX ? process.env.USE_HTX === "true" : process.env.NODE_ENV !== "test",
  /** Log level for underlying Warp. */
  WARP_LOG_LEVEL: (process.env.WARP_LOG_LEVEL ?? "info") as Parameters<typeof LoggerFactory.INST.logLevel>[0],
  /** How many fetches at once should be made to download Bundlr data? FBS stands for "Fetch Batch Size". */
  BUNDLR_FBS: parseInt(process.env.BUNDLR_FBS ?? "40"),
  /** Configurations for Bundlr downloads. */
  DOWNLOAD: {
    /** Download URL for the bundled data. */
    BASE_URL: "https://arweave.net",
    /** Max allowed timeout (milliseconds). */
    TIMEOUT: 50_000,
    /** Max attempts to retry on caught errors. */
    MAX_ATTEMPTS: 5,
    /** Time to sleep (ms) between each attempt. */
    ATTEMPT_SLEEP: 1000,
  },
  /** Logging stuff for the server. */
  LOG: {
    LEVEL: "info" satisfies LogLevel as LogLevel, // for some reason, :LogLevel doesnt work well
    REDACT: [
      "reqId",
      "res.remoteAddress",
      "res.remotePort",
      "res.hostname",
      "req.remoteAddress",
      "req.remotePort",
      "req.hostname",
    ] as string[],
  },
} as const;
