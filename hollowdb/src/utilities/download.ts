import axios from "axios";
import config from "../configurations";

/** Downloads an object from Bundlr network w.r.t transaction id.
 *
 * @param txid transaction ID on Arweave
 * @template V type of the value
 * @returns unbundled raw value
 */
export async function downloadFromBundlr<V>(txid: string) {
  // if `USE_HTX`, it means that values are stored as `hash.txId` (as a string)
  // so to get the txid we split by `.` and then get the second element
  if (config.USE_HTX) {
    txid = txid.split(".")[1];
  }
  const url = `${config.DOWNLOAD.BASE_URL}/${txid}`;
  // const response = await fetch(url);
  const response = await axios.get(url, {
    timeout: config.DOWNLOAD.TIMEOUT,
  });
  if (response.status !== 200) {
    throw new Error(`Bundlr failed with ${response.status}`);
  }

  return response.data as V;
}
