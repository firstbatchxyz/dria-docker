import type { RouteHandler } from "fastify";
import type { Get, GetMany } from "../schemas";
import { RocksdbClient } from "../clients/rocksdb";
import configurations from "../configurations";

export const get: RouteHandler<{ Params: Get }> = async ({ server, params }) => {
  const value = await server.hollowdb.get(params.key);
  return { value };
};

export const getMany: RouteHandler<{ Params: GetMany }> = async ({ server, params }) => {
  const values = await server.hollowdb.getMany(params.keys);
  return { values };
};

export const getRaw: RouteHandler<{ Params: Get }> = async ({ server, params }) => {
  const rocksdb = new RocksdbClient(server.rocksdbPath, server.hollowdb.contractTxId);

  await rocksdb.open();
  const value = await rocksdb.get(params.key);
  await rocksdb.close();

  return { value };
};

export const getManyRaw: RouteHandler<{ Params: GetMany }> = async ({ server, params }) => {
  const rocksdb = new RocksdbClient(configurations.ROCKSDB_PATH, server.hollowdb.contractTxId);

  await rocksdb.open();
  const values = await rocksdb.getMany(params.keys);
  await rocksdb.close();

  return { values };
};
