import initialState from "../res/initialState";
import contractSource from "../res/contractSource";
import { randomUUID } from "crypto";
import { loremIpsum } from "lorem-ipsum";
import { JWKInterface, WarpFactory, Warp } from "warp-contracts";
import { DeployPlugin } from "warp-contracts-plugin-deploy";
import { CacheTypes } from "../../src/types";
/**
 * Returns the size of a given data in bytes.
 * - To convert to KBs: `size / (1 << 10)`
 * - To convert to MBs: `size / (1 << 20)`
 * @param data data, such as `JSON.stringify(body)` for a POST request.
 * @returns data size in bytes
 */
export function size(data: string) {
  return new Blob([data]).size;
}

/** A tiny API wrapper. */
export class FetchClient {
  constructor(readonly baseUrl: string) {}

  /**
   * Generic POST utility for HollowDB micro. Depending on the
   * request, call `response.json()` or `response.text()` to parse
   * the returned body.
   * @param url url
   * @param data body
   * @returns response object
   */
  async post<Body = unknown>(url: string, data?: Body) {
    const body = JSON.stringify(data ?? {});
    return fetch(this.baseUrl + url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json; charset=utf-8",
      },
      body,
    });
  }
}

/**
 * Creates a local warp instance, also uses the `DeployPlugin`.
 *
 * WARNING: Do not use `useStateCache` and `useContractCache` together with `forLocal`.
 */
export function makeLocalWarp(port: number, caches: CacheTypes): Warp {
  return WarpFactory.forLocal(port).use(new DeployPlugin()).useKVStorageFactory(caches.kvFactory);
}

/** Returns a random key-value pair related to our internal usage. */
export function randomKeyValue(options?: { numVals?: number; numChildren?: number }): {
  key: string;
  value: {
    children: number[];
    f: number;
    is_leaf: boolean;
    n_descendants: number;
    metadata: {
      text: string;
    };
    v: number[];
  };
} {
  const numChildren = options?.numChildren || Math.round(Math.random() * 5);
  const numVals = options?.numVals || Math.round(Math.random() * 500 + 100);

  return {
    key: randomUUID(),
    value: {
      children: Array.from({ length: numChildren }, () => Math.round(Math.random() * 50)),
      f: Math.round(Math.random() * 100),
      is_leaf: Math.random() < 0.5,
      metadata: {
        text: loremIpsum({ count: 4 }),
      },
      n_descendants: Math.round(Math.random() * 50),
      v: Array.from({ length: numVals }, () => Math.random() * 2 - 1),
    },
  };
}

/**
 * Deploy a new contract via the provided Warp instance.
 * @param owner owner wallet
 * @param warp a `Warp` instance
 *  */
export async function deploy(
  owner: JWKInterface,
  warp: Warp
): Promise<{ contractTxId: string; srcTxId: string | undefined }> {
  if (warp.environment !== "local") {
    throw new Error("Expected a local Warp environment.");
  }

  const { contractTxId, srcTxId } = await warp.deploy(
    {
      wallet: owner,
      initState: JSON.stringify(initialState),
      src: contractSource,
      evaluationManifest: {
        evaluationOptions: {
          allowBigInt: true,
          useKVStorage: true,
        },
      },
    },
    true // disable bundling in test environment
  );

  return { contractTxId, srcTxId };
}
