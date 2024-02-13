import { makeServer } from "./server";

makeServer().then(async (server) => {
  const addr = await server.listen({ port: 8080 });
  console.log(`Listening at: ${addr}`);
});
