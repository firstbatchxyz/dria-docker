import axios from "axios";
import configurations from "../configurations";
import { FastifyBaseLogger } from "fastify";
import { sleep } from "warp-contracts";

/** Downloads an object from Bundlr network w.r.t transaction id.
 *
 * If USE_HTX is enabled, it means that values are stored as `hash.txId` (as a string),
 * so to get the txid we split by `.` and then get the second element.
 *
 * @param txid transaction ID on Arweave
 * @template V type of the value
 * @returns unbundled raw value
 */
export async function downloadFromBundlr<V>(txid: string, log: FastifyBaseLogger) {
  if (configurations.USE_HTX) {
    const split = txid.split(".");
    if (split.length !== 2) {
      log.warn("Expected value to be a hash.txid, received: " + txid);
    }
    txid = split[1];
  }

  const url = `${configurations.DOWNLOAD.BASE_URL}/${txid}`;

  // try for a few times
  for (let a = 0; a <= configurations.DOWNLOAD.MAX_ATTEMPTS; ++a) {
    try {
      const response = await axios.get(url, {
        timeout: configurations.DOWNLOAD.TIMEOUT,
      });
      if (response.status !== 200) {
        throw new Error(`Bundlr failed with ${response.status}`);
      }
      return response.data as V;
    } catch (err) {
      if (a === configurations.DOWNLOAD.MAX_ATTEMPTS) {
        throw err;
      } else {
        log.warn(
          `(tries: ${a + 1}/${configurations.DOWNLOAD.MAX_ATTEMPTS})` +
            `\tError downloading ${url}: ${(err as Error).message}`
        );
        await sleep(configurations.DOWNLOAD.ATTEMPT_SLEEP);
      }
    }
  }

  throw new Error("All attempts failed.");
}

/** Returns a pretty string about the current download progress.
 * @param cur current value, can be more than `max`
 * @param max maximum value
 * @param decimals (optional) number of decimals for the percentage (default: 2)
 * @returns progress description
 */
export function progressString(cur: number, max: number, decimals: number = 2) {
  const val = Math.min(cur, max);
  const percentage = (val / max) * 100;
  return `[${val} / ${max}] (${percentage.toFixed(decimals)}%)`;
}

/**
 * Map a given key to a value key.
 * @param contractTxId contract txID
 * @param key key
 * @returns value key
 */
export function toValueKey(contractTxId: string, key: string) {
  return `${contractTxId}.value.${key}`;
}

/**
 * Map a given key to a sortKey key.
 * @param contractTxId contract txID
 * @param key key
 * @returns sortKey key
 */
export function toSortKeyKey(contractTxId: string, key: string) {
  return `${contractTxId}.sortKey.${key}`;
}
