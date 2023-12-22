import type { LoggerFactory } from "warp-contracts";

export default {
  /** Redis URL to connect to. Defaults to `redis://default:redispw@localhost:6379`. */
  REDIS_URL: process.env.REDIS_URL || "redis://default:redispw@localhost:6379",
  /** Path to Arweave wallet. */
  WALLET_PATH: process.env.WALLET_PATH || "./config/wallet.json",
  /** Treat values as Bundlr txIds. */
  USE_BUNDLR: process.env.USE_BUNDLR === "true",
  /** Use the optimized [`hash.txid`](https://github.com/firstbatchxyz/hollowdb/blob/master/src/contracts/hollowdb-htx.contract.ts) layout for values. */
  USE_HTX: process.env.USE_HTX === "true",
  /** Log level for underlying Warp. */
  WARP_LOG_LEVEL: (process.env.WARP_LOG_LEVEL || "info") as Parameters<typeof LoggerFactory.INST.logLevel>[0],
  /** Arweave port for `arlocal`. */
  ARWEAVE_PORT: 3169,
  /** How many fetches at once should be made to download Bundlr data? FBS stands for "Fetch Batch Size". */
  BUNDLR_FBS: parseInt(process.env.BUNDLR_FBS || "40"),
  /** Base URL for Bundlr downloads. */
  DOWNLOAD_BASE_URL: "https://arweave.net",
} as const;
