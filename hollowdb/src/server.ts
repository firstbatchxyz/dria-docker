import fastify from "fastify";
import { TypeBoxTypeProvider } from "@fastify/type-provider-typebox";
import { get, getMany, getManyRaw, getRaw, state } from "./controllers/read";
import { put, putMany, remove, set, setMany, update } from "./controllers/write";
import { clear, refresh } from "./controllers/values";
import { Clear, Get, GetMany, Put, PutMany, Remove, Set, SetMany, Update } from "./schemas";
import { SetSDK } from "hollowdb";

export async function makeServer(hollowdb: SetSDK<any>, rocksdbPath: string) {
  const server = fastify().withTypeProvider<TypeBoxTypeProvider>();

  server.decorate("hollowdb", hollowdb);
  server.decorate("contractTxId", hollowdb.contractTxId);
  // TODO: store RocksDB itself here maybe?
  server.decorate("rocksdbPath", rocksdbPath);

  // TODO: put on listen handler here for refresh

  server.get("/state", state);
  server.get("/get", { schema: { params: Get } }, get);
  server.get("/getMany", { schema: { params: GetMany } }, getMany);
  server.get("/getRaw", { schema: { params: Get } }, getRaw);
  server.get("/getManyRaw", { schema: { params: GetMany } }, getManyRaw);

  server.post("/put", { schema: { body: Put } }, put);
  server.post("/putMany", { schema: { body: PutMany } }, putMany);
  server.post("/set", { schema: { body: Set } }, set);
  server.post("/setMany", { schema: { body: SetMany } }, setMany);
  server.post("/update", { schema: { body: Update } }, update);
  server.post("/remove", { schema: { body: Remove } }, remove);

  server.post("/clear", { schema: { body: Clear } }, clear);
  server.post("/refresh", refresh);

  return server;
}
