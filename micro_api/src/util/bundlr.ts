import axios from "axios";
import config from "../config";

/** Downloads an object from Bundlr network w.r.t transaction id. */
export async function downloadFromBundlr<V>(txid: string) {
  // if `USE_HTX`, it means that values are stored as `hash.txId` (as a string)
  // so to get the txid we split by `.` and then get the second element
  if (config.USE_HTX) {
    txid = txid.split(".")[1];
  }
  const url = `${config.DOWNLOAD_BASE_URL}/${txid}`;
  // const response = await fetch(url);
  const response = await axios.get(url, {
    timeout: 50_000, // 50 sec timeout
  });
  if (response.status !== 200) {
    throw new Error(`Bundlr failed with ${response.status}`);
  }

  return response.data as V;
}

//// bundlr uploader for future work
// export async function upload(jwk, payload: T) {
//   const bundlr = new Bundlr.default('http://node1.bundlr.network', 'arweave', jwk);
//   const tags = [{name: 'Content-Type', value: 'application/json'}];
//   const transaction = await bundlr.createTransaction(
//     JSON.stringify({
//       data: payload,
//     }),
//     {
//       tags: tags,
//     }
//   );

//   await transaction.sign();
//   const txID = transaction.id;

//   // you can choose to not await this if you want to upload in the background
//   // but if the upload fails, you will not be able to get the data from the txid
//   await transaction.upload();

//   return txID;
// }
