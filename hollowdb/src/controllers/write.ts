import type { RouteHandler } from "fastify";
import type { Put, PutMany, Remove, Set, SetMany, Update } from "../schemas";

export const put: RouteHandler<{ Body: Put }> = async ({ server, body }, reply) => {
  await server.hollowdb.put(body.key, body.value);
};

export const putMany: RouteHandler<{ Body: PutMany }> = async ({ server, body }, reply) => {
  await server.hollowdb.putMany(body.keys, body.values);
};

export const set: RouteHandler<{ Body: Set }> = async ({ server, body }, reply) => {
  await server.hollowdb.set(body.key, body.value);
};

export const setMany: RouteHandler<{ Body: SetMany }> = async ({ server, body }, reply) => {
  await server.hollowdb.setMany(body.keys, body.values);
};

export const update: RouteHandler<{ Body: Update }> = async ({ server, body }, reply) => {
  await server.hollowdb.update(body.key, body.value, body.proof);
};

export const remove: RouteHandler<{ Body: Remove }> = async ({ server, body }, reply) => {
  await server.hollowdb.remove(body.key, body.proof);
};
