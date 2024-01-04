import { WarpFactory, Warp } from "warp-contracts";
import type { CacheTypes } from "../types/caches";

/** Creates a Warp instance connected to mainnet. */
export function makeWarp(caches: CacheTypes): Warp {
  return WarpFactory.forMainnet()
    .useStateCache(caches.state)
    .useContractCache(caches.contract, caches.src)
    .useKVStorageFactory(caches.kvFactory);
}
