import type { CacheOptions } from "warp-contracts";
import type { Redis } from "ioredis";
import type { RedisOptions } from "warp-contracts-redis";
import type { CacheTypes } from "../types";
import { RedisCache } from "warp-contracts-redis";
import config from "../config";

/**
 * Utility to create Warp Redis caches.
 *
 * @param contractTxId contract transaction id to be used as prefix in the keys
 * @param client optional client to used a self-managed cache, i.e. you are responsible from
 * opening and closing the client.
 * @returns caches
 */
export function createCaches(contractTxId: string, client?: Redis): CacheTypes<RedisCache> {
  const defaultCacheOptions: CacheOptions = {
    inMemory: true,
    subLevelSeparator: "|",
    dbLocation: "redis.micro",
  };

  // if a client exists, use it; otherwise connect via URL
  const redisOptions: RedisOptions = client ? { client } : { url: config.REDIS_URL };

  return {
    state: new RedisCache(
      {
        ...defaultCacheOptions,
        dbLocation: `${contractTxId}.state`,
      },
      redisOptions
    ),
    contract: new RedisCache(
      {
        ...defaultCacheOptions,
        dbLocation: `${contractTxId}.contract`,
      },
      redisOptions
    ),
    src: new RedisCache(
      {
        ...defaultCacheOptions,
        dbLocation: `${contractTxId}.src`,
      },
      redisOptions
    ),
    kvFactory: (contractTxId: string) =>
      new RedisCache(
        {
          ...defaultCacheOptions,
          dbLocation: `${contractTxId}.kv`,
        },
        redisOptions
      ),
  };
}
