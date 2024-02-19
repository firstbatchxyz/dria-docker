import type { FastifyInstance } from "fastify";
import type { Redis } from "ioredis";
import { lastPossibleSortKey } from "warp-contracts";
import type { KeyedSortKeyCacheResult } from "../types";
import { downloadFromBundlr, progressString, toSortKeyKey } from "./download";
import { RocksdbClient } from "../clients/rocksdb";
import configurations from "../configurations";

/**
 * Refresh keys.
 * @param server HollowDB server
 * @returns number of refreshed keys
 */
export async function refreshKeys(server: FastifyInstance): Promise<number> {
  server.log.info(`\nRefreshing keys (${server.hollowdb.contractTxId})\n`);
  await server.hollowdb.base.readState(); // get to the latest state

  const kv = server.hollowdb.base.warp.kvStorageFactory(server.hollowdb.contractTxId);
  const redis = kv.storage<Redis>();

  // get all keys
  const keys = await kv.keys(lastPossibleSortKey);

  // return early if there are no keys
  if (keys.length === 0) {
    server.log.info("All keys are up-to-date.");
    return 0;
  }

  // get the last sortKey for each key
  const sortKeyCacheResults = await Promise.all(keys.map((key) => kv.getLast(key)));

  // from these values, get the ones that are out-of-date (i.e. stale)
  const latestSortKeys: (string | null)[] = sortKeyCacheResults.map((skcr) => (skcr ? skcr.sortKey : null));
  const existingSortKeys: (string | null)[] = await redis.mget(
    ...keys.map((key) => toSortKeyKey(server.contractTxId, key))
  );
  const staleResults: KeyedSortKeyCacheResult[] = sortKeyCacheResults
    .map((skcr, i) =>
      // filter out existing sortKeys
      // also store the respective `key` with the result
      latestSortKeys[i] !== existingSortKeys[i] ? { sortKeyCacheResult: skcr, key: keys[i] } : null
    )
    // this filter will filter out both existing null values, and matching sortKeys
    .filter((res): res is KeyedSortKeyCacheResult => res !== null);

  // return early if everything is up-to-date
  if (staleResults.length === 0) {
    return 0;
  }

  const rocksdb = new RocksdbClient(server.rocksdbPath, server.contractTxId);
  await rocksdb.open();

  const refreshValues = async (results: KeyedSortKeyCacheResult[], values?: unknown[]) => {
    if (values && values.length !== results.length) {
      throw new Error("array length mismatch");
    }

    // create [key, value] pairs for with stringified values
    const valuePairs = results.map(({ key, sortKeyCacheResult: { cachedValue } }, i) => {
      const val = values
        ? // override with given value
          typeof values[i] === "string"
          ? (values[i] as string)
          : JSON.stringify(values[i])
        : // use own value
          JSON.stringify(cachedValue);
      return [key, val] as [string, string];
    });

    // write values to disk (as they may be too much for the memory)
    await rocksdb.setMany(valuePairs);

    // store the `sortKey`s for later refreshes to see if a `value` is stale
    const sortKeyPairs = results.map(
      ({ key, sortKeyCacheResult: { sortKey } }) =>
        [toSortKeyKey(server.contractTxId, key), sortKey] as [string, string]
    );
    await redis.mset(...sortKeyPairs.flat());
  };

  // update values in Redis

  const { USE_BUNDLR, BUNDLR_FBS } = configurations;
  if (USE_BUNDLR) {
    const progress: [number, number] = [0, 0];

    server.log.info("Starting batched Bundlr downloads:");
    progress[1] = staleResults.length;
    for (let b = 0; b < staleResults.length; b += BUNDLR_FBS) {
      const batchResults = staleResults.slice(b, b + BUNDLR_FBS);

      progress[0] = Math.min(b + BUNDLR_FBS, staleResults.length);

      const startTime = performance.now();
      const batchValues = await Promise.all(
        batchResults.map((result) =>
          downloadFromBundlr<{ data: any }>(result.sortKeyCacheResult.cachedValue as string, server.log)
        )
      );
      const endTime = performance.now();
      server.log.info(
        `${progressString(progress[0], progress[1])} values downloaded (${(endTime - startTime).toFixed(2)} ms)`
      );

      await refreshValues(
        batchResults,
        // our Bundlr service uploads as "{data: payload}" so we parse it here
        batchValues.map((val) => val.data)
      );
    }
    server.log.info("Downloaded & refreshed all stale values.");
  } else {
    await refreshValues(staleResults);
  }

  await rocksdb.close();

  return staleResults.length;
}
