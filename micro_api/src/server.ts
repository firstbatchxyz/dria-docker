import { json, createError, send } from "micro";
import { IncomingMessage, ServerResponse } from "http";
import { SDK } from "hollowdb";
import { StatusCodes } from "http-status-codes";
import { LoggerFactory, lastPossibleSortKey } from "warp-contracts";
import { Redis } from "ioredis";

import type { KeyedSortKeyCacheResult, Request } from "./types";
import config from "./config";
import { downloadFromBundlr } from "./util";

/**
 * A higher-order function to create a micro server that uses the given HollowDB instance
 * @param hollowdb instance of hollowdb
 * @param contractTxId connected contract tx id
 * @template V type of the value stored in hollowdb
 * @returns a `micro` server, should be exported via `module.exports`
 */
export default function makeServer<V = unknown>(hollowdb: SDK<V>, contractTxId: string) {
  let isReady = false;

  const kv = hollowdb.base.warp.kvStorageFactory(contractTxId);
  const redis = kv.storage<Redis>();

  /** Maps a key to its `value` key in Redis. */
  const toValueKey = (key: string) => `${contractTxId}.value.${key}`;
  /** Maps a key to its `sortKey` key in Redis. */
  const toSortKeyKey = (key: string) => `${contractTxId}.sortKey.${key}`;
  /** Given keys, sortKeys and values, `mset`s them to their respective keys. Can provide optional value overrides. */
  const refreshRedis = async (results: KeyedSortKeyCacheResult<V>[], values?: V[]) => {
    if (values && values.length !== results.length) {
      throw new Error("array length mismatch");
    }

    // store the `value`s itself within the cache for each `key`
    const valuePairs = results.map(({ key, sortKeyCacheResult: { cachedValue } }, i) => [
      toValueKey(key),
      JSON.stringify(values ? values[i] : cachedValue),
    ]);
    await redis.mset(...valuePairs.flat());

    // store the `sortKey`s for later refreshes to see if a `value` is stale
    const sortKeyPairs = results.map(({ key, sortKeyCacheResult: { sortKey } }) => [toSortKeyKey(key), sortKey]);
    await redis.mset(...sortKeyPairs.flat());
  };

  /** Refresh the cache with values.
   * @returns number of refreshed keys
   */
  async function refresh(): Promise<number> {
    console.log(`\nRefreshing keys (${contractTxId})\n`);

    // refreshes to the latest state
    await hollowdb.base.readState();

    // find the latest sortKey & value for each `key` in the cache
    const keys = await kv.keys(lastPossibleSortKey);

    // return early if there are no keys
    if (keys.length === 0) {
      console.log(`No keys found to refresh.`);
      return 0;
    }

    const sortKeyCacheResults = await Promise.all(keys.map((key) => kv.getLast(key)));

    // from these values, get the ones that are out-of-date (i.e. stale)
    const latestSortKeys: (string | null)[] = sortKeyCacheResults.map((skcr) => (skcr ? skcr.sortKey : null));
    const existingSortKeys: (string | null)[] = await redis.mget(...keys.map((key) => toSortKeyKey(key)));
    const staleResults: KeyedSortKeyCacheResult<V>[] = sortKeyCacheResults
      .map((skcr, i) =>
        // also store the respective `key` with the result
        latestSortKeys[i] !== existingSortKeys[i] ? { sortKeyCacheResult: skcr, key: keys[i] } : null
      )
      // this filter will filter out both existing null values, and matching sortKeys
      .filter((res): res is KeyedSortKeyCacheResult<V> => res !== null);

    // return early if everything is up-to-date
    if (staleResults.length === 0) {
      return 0;
    }

    // update values in Redis
    if (config.USE_BUNDLR) {
      // doing a Promise.all over all keys here can be problematic, causes timeouts and stuff
      // (see relevant issue here: https://github.com/firstbatchxyz/hollowdb-dockerized/issues/7)
      // so instead we do these fetches batch-by-batch
      console.log("Starting batched Bundlr downloads:");
      for (let b = 0; b < staleResults.length; b += config.BUNDLR_FBS) {
        const batchResults = staleResults.slice(b, b + config.BUNDLR_FBS);

        const msg = `\t[${b} of ${staleResults.length} values downloaded]`;
        console.time(msg);
        const batchValues: V[] = await Promise.all(
          batchResults.map((result) => downloadFromBundlr<V>(result.sortKeyCacheResult.cachedValue as string))
        );
        console.timeEnd(msg);

        await refreshRedis(batchResults, batchValues);
      }
      console.log("Downloaded & refreshed all stale values.");
    } else {
      await refreshRedis(staleResults);
    }

    return staleResults.length;
  }

  // sync to the latest on-chain state & refresh
  refresh().then(() => {
    isReady = true;
    console.log("Server synced & ready.");
    console.log("> Config:\n", config);
    console.log(`> Redis: ${config.REDIS_URL}`);
    console.log(`> Wallet: ${config.WALLET_PATH}`);
    console.log(`> Download URL: ${config.DOWNLOAD_BASE_URL}`);
    console.log(`> Contract: https://sonar.warp.cc/#/app/contract/${contractTxId}`);
  });
  LoggerFactory.INST.logLevel(config.WARP_LOG_LEVEL);

  return async (req: IncomingMessage, res: ServerResponse): Promise<void> => {
    if (!isReady) {
      return send(res, StatusCodes.SERVICE_UNAVAILABLE, "Cache is still loading, try again shortly.");
    }

    // parse the request, it is either a (GET) "/key" or (POST)
    const url = req.url || "/";
    const reqBody: Request<V> =
      url === "/"
        ? // this is a POST request with JSON body
          ((await json(req)) as Request<V>)
        : // this is a GET request
          // in our case, the url itself should be the key
          {
            route: "GET",
            data: { key: url.slice(url.lastIndexOf("/") + 1) },
          };

    const { route, data } = reqBody;
    // console.log({ route, data });
    try {
      switch (route) {
        case "GET": {
          const value = await hollowdb.get(data.key);
          return send(res, StatusCodes.OK, { value });
        }
        case "GET_RAW": {
          const rawValue = await redis.get(toValueKey(data.key));
          const value = rawValue ? JSON.parse(rawValue) : null;
          return send(res, StatusCodes.OK, { value });
        }
        case "GET_MANY": {
          const values = await hollowdb.getMany(data.keys);
          return send(res, StatusCodes.OK, { values });
        }
        case "GET_MANY_RAW": {
          const rawValues = await redis.mget(...data.keys.map((key) => toValueKey(key)));
          const values = rawValues.map((rawValue) => (rawValue ? JSON.parse(rawValue) : null));
          return send(res, StatusCodes.OK, { values });
        }
        case "PUT": {
          await hollowdb.put(data.key, data.value);
          return send(res, StatusCodes.OK);
        }
        case "PUT_MANY": {
          if (data.keys.length !== data.values.length) {
            return send(res, StatusCodes.BAD_REQUEST, "Keys and values count do not match.");
          }
          await hollowdb.putMany(data.keys, data.values);
          return send(res, StatusCodes.OK);
        }
        case "UPDATE": {
          await hollowdb.update(data.key, data.value, data.proof);
          return send(res, StatusCodes.OK);
        }
        case "REMOVE": {
          await hollowdb.remove(data.key, data.proof);
          return send(res, StatusCodes.OK);
        }
        case "STATE": {
          const state = await hollowdb.getState();
          return send(res, StatusCodes.OK, state);
        }
        case "REFRESH": {
          const numStaleResults = await refresh();
          return send(res, StatusCodes.OK, numStaleResults);
        }
        case "CLEAR": {
          const keys = data.keys ? data.keys : await kv.keys(lastPossibleSortKey);

          await redis.del(...keys.map((key) => toSortKeyKey(key)));
          await redis.del(...keys.map((key) => toValueKey(key)));

          return send(res, StatusCodes.OK, keys.length);
        }
        default:
          route satisfies never;
          return send(res, StatusCodes.NOT_FOUND, "Unknown route.");
      }
    } catch (err) {
      console.error(err);

      const error = err as Error;
      if (error.message.startsWith("Contract Error")) {
        return send(res, StatusCodes.BAD_REQUEST, error.message);
      }

      throw createError(StatusCodes.INTERNAL_SERVER_ERROR, error.message, error);
    }
  };
}
