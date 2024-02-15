import type { SortKeyCacheResult } from "warp-contracts";
import type { RedisCache } from "warp-contracts-redis";

/** Cache types used by `Warp`. */
export type CacheTypes<C = RedisCache> = {
  state: C;
  contract: C;
  src: C;
  kvFactory: (contractTxId: string) => C;
};

/**
 * A `SortKeyCacheResult` with its respective `key` attached to it.
 *
 * - `key` is accessed as `.key`
 * - `sortKey` is accessed as `.sortKeyCacheResult.sortKey`
 * - `value` is accessed as `.sortKeyCacheResult.value`
 */
export type KeyedSortKeyCacheResult<V = unknown> = { sortKeyCacheResult: SortKeyCacheResult<V>; key: string };
