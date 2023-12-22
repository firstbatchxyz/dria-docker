import { randomUUID } from "crypto";
import { loremIpsum } from "lorem-ipsum";

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
