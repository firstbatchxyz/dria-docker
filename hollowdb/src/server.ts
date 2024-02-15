import fastify, { LogLevel } from "fastify";
import type { TypeBoxTypeProvider } from "@fastify/type-provider-typebox";
import { get, getMany, getManyRaw, getRaw, state } from "./controllers/read";
import { put, putMany, remove, set, setMany, update } from "./controllers/write";
import { clear, refresh } from "./controllers/values";
import { Clear, Get, GetMany, Put, PutMany, Remove, Set, SetMany, Update } from "./schemas";
import { SetSDK } from "hollowdb";
import { LoggerFactory } from "warp-contracts";

export async function makeServer(hollowdb: SetSDK<any>, rocksdbPath: string) {
  const logLevel: LogLevel = "silent";
  const server = fastify({
    logger: {
      level: logLevel,
      transport: {
        target: "pino-pretty",
      },
    },
  }).withTypeProvider<TypeBoxTypeProvider>();
  LoggerFactory.INST.logLevel("debug");

  server.decorate("hollowdb", hollowdb);
  server.decorate("contractTxId", hollowdb.contractTxId);
  server.decorate("rocksdbPath", rocksdbPath); // TODO: store RocksDB itself here maybe?

  server.get("/state", state);

  server.post("/get", { schema: { body: Get } }, get);
  server.post("/getRaw", { schema: { body: Get } }, getRaw);

  server.post("/getMany", { schema: { body: GetMany } }, getMany);
  server.post("/getManyRaw", { schema: { body: GetMany } }, getManyRaw);

  server.post("/put", { schema: { body: Put } }, put);
  server.post("/putMany", { schema: { body: PutMany } }, putMany);

  server.post("/set", { schema: { body: Set } }, set);
  server.post("/setMany", { schema: { body: SetMany } }, setMany);

  server.post("/update", { schema: { body: Update } }, update);

  server.post("/remove", { schema: { body: Remove } }, remove);

  server.post("/clear", { schema: { body: Clear } }, clear);

  server.post("/refresh", refresh);

  server.addHook("onError", (request, reply, error, done) => {
    if (error.message.startsWith("Contract Error")) {
      reply.status(400);
    }
    done();
  });

  return server;
}
