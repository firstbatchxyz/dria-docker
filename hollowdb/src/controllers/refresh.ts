import { RouteHandler } from "fastify";

export const refresh: RouteHandler = (request, reply) => {};

export const clear: RouteHandler = () => {};

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
