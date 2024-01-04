import { WarpFactory, Warp } from "warp-contracts";
import type { CacheTypes } from "../types/caches";
import { DeployPlugin } from "warp-contracts-plugin-deploy";
import { SnarkjsExtension } from "warp-contracts-plugin-snarkjs";
import { EthersExtension } from "warp-contracts-plugin-ethers";

/**
 * Creates a local warp instance, also uses the `DeployPlugin`.
 *
 * WARNING: Do not use `useStateCache` and `useContractCache` together with
 * `forLocal`.
 */
export function makeLocalWarp(port: number, caches?: CacheTypes): Warp {
  let warp = WarpFactory.forLocal(port).use(new DeployPlugin()).use(new SnarkjsExtension()).use(new EthersExtension());
  if (caches) {
    warp = warp.useKVStorageFactory(caches.kvFactory);
  }
  return warp;
}
