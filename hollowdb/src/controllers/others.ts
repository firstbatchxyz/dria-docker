import { RouteHandler } from "fastify";
import type { Clear } from "../schemas";
import type { Redis } from "ioredis";
import { lastPossibleSortKey } from "warp-contracts";
import { RocksdbClient } from "../clients/rocksdb";

const toSortKeyKey = (contractTxId: string, key: string) => `${contractTxId}.sortKey.${key}`;

export const state: RouteHandler = async ({ server }, reply) => {
  return await server.hollowdb.getState();
};

export const refresh: RouteHandler = (request, reply) => {};

export const clear: RouteHandler<{ Body: Clear }> = async ({ server, body }, reply) => {
  const kv = server.hollowdb.base.warp.kvStorageFactory(server.hollowdb.contractTxId);
  const redis = kv.storage<Redis>();

  const keys = body.keys ? body.keys : await kv.keys(lastPossibleSortKey);

  await redis.del(...keys.map((key) => toSortKeyKey(server.hollowdb.contractTxId, key)));

  const rocksdb = new RocksdbClient(server.rocksdbPath, server.hollowdb.contractTxId);
  await rocksdb.open();
  await rocksdb.removeMany(keys);
  await rocksdb.close();
};

/** Returns a pretty string about the current progress.
 * @param cur current value, can be more than `max`
 * @param max maximum value
 * @param decimals (optional) number of decimals for the percentage (default: 2)
 * @returns progress description
 */
function progressString(cur: number, max: number, decimals: number = 2) {
  const val = Math.min(cur, max);
  const percentage = (val / max) * 100;
  return `[${val} / ${max}] (${percentage.toFixed(decimals)}%)`;
}
