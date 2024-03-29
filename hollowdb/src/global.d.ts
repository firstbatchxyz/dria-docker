import fastify from "fastify";
import { SetSDK } from "hollowdb";
import http from "http";

declare module "fastify" {
  export interface FastifyInstance<
    HttpServer = http.Server,
    HttpRequest = http.IncomingMessage,
    HttpResponse = http.ServerResponse
  > {
    /** HollowDB decorator. */
    hollowdb: SetSDK<any>;
    /** Contract ID. */
    contractTxId: string;
    /** RocksDB Path. */
    rocksdbPath: string;
  }
}
