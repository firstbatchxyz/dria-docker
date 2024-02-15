import type { RouteHandler } from "fastify";
import type { Get, GetMany } from "../schemas";
import { RocksdbClient } from "../clients/rocksdb";
import configurations from "../configurations";

export const get: RouteHandler<{ Body: Get }> = async ({ server, body }) => {
  const value = await server.hollowdb.get(body.key);
  return { value };
};

export const getRaw: RouteHandler<{ Body: Get }> = async ({ server, body }) => {
  const rocksdb = new RocksdbClient(server.rocksdbPath, server.hollowdb.contractTxId);

  await rocksdb.open();
  const value = await rocksdb.get(body.key);
  await rocksdb.close();

  return { value };
};

export const getMany: RouteHandler<{ Body: GetMany }> = async ({ server, body }) => {
  const values = await server.hollowdb.getMany(body.keys);
  return { values };
};

export const getManyRaw: RouteHandler<{ Body: GetMany }> = async ({ server, body }) => {
  const rocksdb = new RocksdbClient(configurations.ROCKSDB_PATH, server.hollowdb.contractTxId);

  console.log("sponge 1");
  await rocksdb.open();
  console.log("sponge 2");
  const values = await rocksdb.getMany(body.keys);
  console.log("sponge 3");
  await rocksdb.close();
  console.log("sponge 4");

  return { values };
};

export const state: RouteHandler = async ({ server }) => {
  return await server.hollowdb.getState();
};
