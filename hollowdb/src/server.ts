import fastify from "fastify";
import { get, getMany, getManyRaw, getRaw } from "./controllers/read";
import { put, putMany, remove, update } from "./controllers/write";

export async function makeServer() {
  const server = fastify();

  server.get("/ping", async () => "pong\n");
  server.post("/state", (request, response) => ({}));

  server.get("/get", get);
  server.get("/getMany", getMany);
  server.get("/getRaw", getRaw);
  server.get("/getManyRaw", getManyRaw);

  server.post("/put", put);
  server.post("/putMany", putMany);
  server.post("/update", update);
  server.post("/remove", remove);

  server.post("/refresh", (request, response) => ({}));
  server.post("/clear", (request, response) => ({}));

  return server;
}
