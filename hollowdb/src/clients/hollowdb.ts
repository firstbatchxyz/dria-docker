import { WarpFactory, Warp } from "warp-contracts";
import type { CacheOptions } from "warp-contracts";
import type { Redis } from "ioredis";
import { RedisCache, type RedisOptions } from "warp-contracts-redis";
import type { CacheTypes } from "../types";

/**
 * Utility to create Warp Redis caches.
 *
 * @param contractTxId contract transaction id to be used as prefix in the keys
 * @param client Redis client to use a self-managed cache
 * @returns caches
 */
export function createCaches(contractTxId: string, client: Redis): CacheTypes<RedisCache> {
  const defaultCacheOptions: CacheOptions = {
    inMemory: true,
    subLevelSeparator: "|",
    dbLocation: "redis.hollowdb",
  };

  // if a client exists, use it; otherwise connect via URL
  const redisOptions: RedisOptions = { client };

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

/** Creates a Warp instance connected to mainnet. */
export function makeWarp(caches: CacheTypes): Warp {
  return WarpFactory.forMainnet()
    .useStateCache(caches.state)
    .useContractCache(caches.contract, caches.src)
    .useKVStorageFactory(caches.kvFactory);
}
