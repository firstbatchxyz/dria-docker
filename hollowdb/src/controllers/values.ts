import { RouteHandler } from "fastify";
import type { Clear } from "../schemas";
import type { Redis } from "ioredis";
import { lastPossibleSortKey } from "warp-contracts";
import { RocksdbClient } from "../clients/rocksdb";
import { toSortKeyKey } from "../utilities/download";
import { refreshKeys } from "../utilities/refresh";

export const refresh: RouteHandler = async ({ server }, reply) => {
  const numKeysRefreshed = await refreshKeys(server);
  return reply.send(numKeysRefreshed);
};

export const clear: RouteHandler<{ Body: Clear }> = async ({ server, body }, reply) => {
  const kv = server.hollowdb.base.warp.kvStorageFactory(server.hollowdb.contractTxId);
  const redis = kv.storage<Redis>();

  // get all existing keys (without sortKey)
  const keys = body.keys ?? (await kv.keys(lastPossibleSortKey));

  // delete the sortKey mappings from Redis
  await redis.del(...keys.map((key) => toSortKeyKey(server.hollowdb.contractTxId, key)));

  // delete the values from Rocksdb
  const rocksdb = new RocksdbClient(server.rocksdbPath, server.hollowdb.contractTxId);
  await rocksdb.open();
  await rocksdb.removeMany(keys);
  await rocksdb.close();

  return reply.send();
};
