import ArLocal from "arlocal";
import { ArWallet, LoggerFactory, sleep } from "warp-contracts";

import { Redis } from "ioredis";
import { SetSDK } from "hollowdb";

import { makeServer } from "../src/server";
import config from "../src/configurations";
import { createCaches } from "../src/clients/hollowdb";
import { deploy, FetchClient, randomKeyValue, makeLocalWarp } from "./util";
import { Get, GetMany, Put, PutMany, Update } from "../src/schemas";
import { randomBytes } from "crypto";
import { rmSync } from "fs";

describe("crud operations", () => {
  let arlocal: ArLocal;
  let redisClient: Redis;
  let client: FetchClient;
  let url: string;

  const DATA_PATH = "./test/data";
  const ARWEAVE_PORT = 3169;
  const VALUE = randomBytes(16).toString("hex");
  const NEW_VALUE = randomBytes(16).toString("hex");
  const KEY = randomBytes(16).toString("hex");

  beforeAll(async () => {
    console.log("Starting...");

    // create a local Arweave instance
    arlocal = new ArLocal(ARWEAVE_PORT, false);
    await arlocal.start();

    // deploy a contract locally and generate a wallet
    redisClient = new Redis(config.REDIS_URL, { lazyConnect: false });
    let caches = createCaches("testing-setup", redisClient);
    let warp = makeLocalWarp(ARWEAVE_PORT, caches);
    const owner: ArWallet = (await warp.generateWallet()).jwk;
    const { contractTxId } = await deploy(owner, warp);

    // start the server & connect to the contract
    caches = createCaches(contractTxId, redisClient);
    warp = makeLocalWarp(ARWEAVE_PORT, caches);
    const hollowdb = new SetSDK(owner, contractTxId, warp);
    const server = await makeServer(hollowdb, `${DATA_PATH}/${contractTxId}`);
    url = await server.listen({ port: config.PORT });
    LoggerFactory.INST.logLevel("none");

    client = new FetchClient(url);

    // TODO: wait a bit due to state syncing
    console.log("waiting a bit for the server to be ready...");
    await sleep(1200);
    console.log("done");
  });

  describe("basic CRUD", () => {
    it("should put & get a value", async () => {
      const putResponse = await client.post<Put>("/put", { key: KEY, value: VALUE });
      expect(putResponse.status).toBe(200);

      const getResponse = await client.post<Get>("/get", { key: KEY });
      expect(getResponse.status).toBe(200);
      expect(await getResponse.json().then((body) => body.value)).toBe(VALUE);
    });

    it("should NOT put to an existing key", async () => {
      const putResponse = await client.post<Put>("/put", { key: KEY, value: VALUE });
      expect(putResponse.status).toBe(400);
      const body = await putResponse.json();
      expect(body.message).toBe("Contract Error [put]: Key already exists.");
    });

    it("should update & get the new value", async () => {
      const updateResponse = await client.post<Update>("/update", { key: KEY, value: NEW_VALUE });
      expect(updateResponse.status).toBe(200);

      const getResponse = await client.post<Get>("/get", { key: KEY });
      expect(getResponse.status).toBe(200);
      expect(await getResponse.json().then((body) => body.value)).toBe(NEW_VALUE);
    });

    it("should remove the new value & get null", async () => {
      const removeResponse = await client.post("/remove", {
        key: KEY,
      });
      expect(removeResponse.status).toBe(200);

      const getResponse = await client.post<Get>("/get", { key: KEY });
      expect(getResponse.status).toBe(200);
      expect(await getResponse.json().then((body) => body.value)).toBe(null);
    });
  });

  describe("batch gets and puts", () => {
    const LENGTH = 10;
    const KEY_VALUES = Array.from({ length: LENGTH }, () => randomKeyValue({ numVals: 768 }));

    it("should put many values", async () => {
      const putResponse = await client.post<PutMany>("/putMany", {
        keys: KEY_VALUES.map((kv) => kv.key),
        values: KEY_VALUES.map((kv) => kv.value),
      });
      expect(putResponse.status).toBe(200);
    });

    it("should get many values", async () => {
      const getManyResponse = await client.post("/getMany", {
        keys: KEY_VALUES.map((kv) => kv.key),
      });
      expect(getManyResponse.status).toBe(200);
      const body = await getManyResponse.json();
      for (let i = 0; i < KEY_VALUES.length; i++) {
        const expected = KEY_VALUES[i].value;
        const result = body.values[i] as (typeof KEY_VALUES)[0]["value"];
        expect(result.metadata.text).toBe(expected.metadata.text);
        expect(result.f).toBe(expected.f);
      }
    });

    it("should refresh the cache for raw GET operations", async () => {
      const refreshResponse = await client.post("/refresh");
      expect(refreshResponse.status).toBe(200);
    });

    it("should do a raw GET", async () => {
      const { key, value } = KEY_VALUES[0];

      const getRawResponse = await client.post<Get>("/getRaw", { key });
      expect(getRawResponse.status).toBe(200);
      const body = await getRawResponse.json();
      const result = body.value as typeof value;
      expect(result.metadata.text).toBe(value.metadata.text);
      expect(result.f).toBe(value.f);
    });

    it("should do a raw GET many", async () => {
      const getManyRawResponse = await client.post<GetMany>("/getManyRaw", {
        keys: KEY_VALUES.map((kv) => kv.key),
      });
      expect(getManyRawResponse.status).toBe(200);
      const body = await getManyRawResponse.json();
      for (let i = 0; i < KEY_VALUES.length; i++) {
        const expected = KEY_VALUES[i].value;
        const result = body.values[i] as (typeof KEY_VALUES)[0]["value"];
        expect(result.metadata.text).toBe(expected.metadata.text);
        expect(result.f).toBe(expected.f);
      }
    });

    it("should refresh a newly PUT key", async () => {
      const { key, value } = randomKeyValue();
      const putResponse = await client.post<Put>("/put", { key, value });
      expect(putResponse.status).toBe(200);

      // we expect only 1 new key to be added via REFRESH
      const refreshResponse = await client.post("/refresh");
      expect(refreshResponse.status).toBe(200);
      expect(await refreshResponse.text()).toBe("1");
    });

    it("should refresh with 0 keys when no additions are made", async () => {
      const refreshResponse = await client.post("/refresh");
      expect(refreshResponse.status).toBe(200);
      expect(await refreshResponse.text()).toBe("0");
    });

    it("should clear all keys", async () => {
      // we expect 0 keys as nothing has changed since the last refresh
      const clearResponse = await client.post("/clear");
      expect(clearResponse.status).toBe(200);

      const getManyRawResponse = await client.post<GetMany>("/getManyRaw", {
        keys: KEY_VALUES.map((kv) => kv.key),
      });
      expect(getManyRawResponse.status).toBe(200);
      const body = await getManyRawResponse.json();

      (body.values as (typeof KEY_VALUES)[0]["value"][]).forEach((val) => expect(val).toBe(null));
    });
  });

  afterAll(async () => {
    console.log("waiting a bit before closing...");
    await sleep(1500);

    rmSync(DATA_PATH, { recursive: true });
    await arlocal.stop();
    await redisClient.quit();
  });
});
