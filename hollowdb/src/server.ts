import fastify from "fastify";
import { TypeBoxTypeProvider } from "@fastify/type-provider-typebox";
import { get, getMany, getManyRaw, getRaw } from "./controllers/read";
import { put, putMany, remove, update } from "./controllers/write";
import { clear, refresh } from "./controllers/others";
import { Clear, Get, GetMany, Put, PutMany, Remove, Update } from "./schemas";
import { SetSDK } from "hollowdb";

export async function makeServer(hollowdb: SetSDK<any>, rocksdbPath: string) {
  const server = fastify().withTypeProvider<TypeBoxTypeProvider>();

  server.decorate("hollowdb", hollowdb);
  server.decorate("rocksdbPath", rocksdbPath);

  server.get("/ping", () => "pong\n");
  server.get("/state", async () => await server.hollowdb.getState());

  server.get("/get", { schema: { params: Get } }, get);
  server.get("/getMany", { schema: { params: GetMany } }, getMany);
  server.get("/getRaw", { schema: { params: Get } }, getRaw);
  server.get("/getManyRaw", { schema: { params: GetMany } }, getManyRaw);

  server.post("/put", { schema: { body: Put } }, put);
  server.post("/putMany", { schema: { body: PutMany } }, putMany);
  server.post("/update", { schema: { body: Update } }, update);
  server.post("/remove", { schema: { body: Remove } }, remove);
  // server.post("/set", { schema: { body: Put } }, put);
  // server.post("/setMany", { schema: { body: PutMany } }, putMany);

  server.post("/refresh", refresh);
  server.post("/clear", { schema: { body: Clear } }, clear);

  return server;
}
