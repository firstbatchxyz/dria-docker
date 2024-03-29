import fastify, { LogLevel } from "fastify";
import type { TypeBoxTypeProvider } from "@fastify/type-provider-typebox";
import { get, getMany, getManyRaw, getRaw, state } from "./controllers/read";
import { put, putMany, remove, set, setMany, update } from "./controllers/write";
import { clear, refresh } from "./controllers/values";
import { Clear, Get, GetMany, Put, PutMany, Remove, Set, SetMany, Update } from "./schemas";
import { SetSDK } from "hollowdb";
import { LoggerFactory } from "warp-contracts";
import configurations from "./configurations";
import { refreshKeys } from "./utilities/refresh";
import { Redis } from "ioredis";

export async function makeServer(hollowdb: SetSDK<any>, rocksdbPath: string) {
  const server = fastify({
    logger: {
      level: configurations.LOG.LEVEL,
      transport: { target: "pino-pretty" },
      redact: {
        paths: configurations.LOG.REDACT,
        remove: true,
      },
    },
  }).withTypeProvider<TypeBoxTypeProvider>();
  LoggerFactory.INST.logLevel(configurations.LOG.LEVEL === "silent" ? "none" : configurations.LOG.LEVEL);

  server.decorate("hollowdb", hollowdb);
  server.decorate("contractTxId", hollowdb.contractTxId);
  server.decorate("rocksdbPath", rocksdbPath); // TODO: store RocksDB itself here maybe?

  // check redis
  await server.hollowdb.base.warp.kvStorageFactory(server.hollowdb.contractTxId).storage<Redis>().ping();

  // refresh keys
  server.log.info("Waiting for cache to be loaded.");
  const numKeysRefreshed = await refreshKeys(server);

  server.log.info(`Server synced & ready! (${numKeysRefreshed} keys refreshed)`);
  server.log.info(`> Redis: ${configurations.REDIS_URL}`);
  server.log.info(`> RocksDB: ${configurations.ROCKSDB_PATH}`);
  server.log.info(`> Download URL: ${configurations.DOWNLOAD.BASE_URL} (timeout ${configurations.DOWNLOAD.TIMEOUT})`);
  server.log.info(`> Contract: https://sonar.warp.cc/#/app/contract/${server.contractTxId}`);

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
