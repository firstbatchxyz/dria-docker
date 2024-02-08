import RocksDB from "rocksdb";

export * from "./warp";
export * from "./bundlr";

/**
 * Given a value, tries to `JSON.parse` it and if parsing fails
 * it will return the value as-is.
 *
 * This is particularly useful when there are some stringified values,
 * and some other non-stringified strings together.
 *
 * @param value a stringified or null value
 * @template V type of the expected value
 * @returns parsed value
 */
export function tryParse<V>(value: RocksDB.Bytes | null): V | null {
  let result = null;

  if (value) {
    try {
      result = JSON.parse(value.toString());
    } catch (err) {
      result = value;
    }
  }

  return result;
}

/** Returns a pretty string about the current progress.
 * @param cur current value, can be more than `max`
 * @param max maximum value
 * @param decimals (optional) number of decimals for the percentage
 * @returns progress description
 */
export function progressString(cur: number, max: number, decimals: number = 2) {
  const val = Math.min(cur, max);
  const percentage = (val / max) * 100;
  return `[${val} / ${max}] (${percentage.toFixed(decimals)}%)`;
}
