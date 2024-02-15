import http from "http";
import ArLocal from "arlocal";
import { ArWallet, LoggerFactory } from "warp-contracts";

import { Redis } from "ioredis";
import { SDK } from "hollowdb";

import { makeServer } from "../src/server";
import config from "../src/configurations";
import { createCaches } from "../src/clients/hollowdb";
import { makeLocalWarp } from "./util";

import { deploy, postData, getKey, sleep, shutdown, randomKeyValue } from "./util";

describe("crud operations", () => {
  let arlocal: ArLocal;
  let service: http.Server;
  let redisClient: Redis;
  let url: string;
  const ARWEAVE_PORT = 3169;

  const VALUE = BigInt(Math.floor(Math.random() * 9_999_999)).toString();
  const NEW_VALUE = BigInt(Math.floor(Math.random() * 9_999_999)).toString();
  const SECRET = BigInt(Math.floor(Math.random() * 9_999_999));
  const KEY = computeKey(SECRET);

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
    const hollowdb = new SDK(owner, contractTxId, warp);
    const server = makeServer(hollowdb, `./test/data/${contractTxId}`, contractTxId);
    service = new http.Server(serve(server));
    url = await listen(service);
    console.log("micro listening at", url);
    LoggerFactory.INST.logLevel("error");

    // dont care about logs for the test
    // LoggerFactory.INST.logLevel("none");

    // TODO: wait a bit due to state syncing
    console.log("waiting a bit for the server to be ready...");
    await sleep(2700);
  });

  describe("basic CRUD", () => {
    it("should put & get a value", async () => {
      const putResponse = await postData(url, {
        route: "PUT",
        data: {
          key: KEY,
          value: VALUE,
        },
      });
      expect(putResponse.status).toBe(200);

      const getResponse = await getKey(url, KEY);
      expect(getResponse.status).toBe(200);
      expect(await getResponse.json().then((body) => body.value)).toBe(VALUE);
    });

    it("should NOT put to an existing key", async () => {
      const putResponse = await postData(url, {
        route: "PUT",
        data: {
          key: KEY,
          value: VALUE,
        },
      });
      expect(putResponse.status).toBe(400);
      expect(await putResponse.text()).toBe("Contract Error [put]: Key already exists.");
    });

    it("should NOT update with a wrong proof", async () => {
      const { proof } = await prover.prove(BigInt(0), VALUE, NEW_VALUE);
      const putResponse = await postData(url, {
        route: "UPDATE",
        data: {
          key: KEY,
          value: NEW_VALUE,
          proof: proof,
        },
      });
      expect(putResponse.status).toBe(400);
      expect(await putResponse.text()).toBe("Contract Error [update]: Invalid proof.");
    });

    it("should update & get the new value", async () => {
      const updateResponse = await postData(url, {
        route: "UPDATE",
        data: {
          key: KEY,
          value: NEW_VALUE,
        },
      });
      expect(updateResponse.status).toBe(200);

      const getResponse = await getKey(url, KEY);
      expect(getResponse.status).toBe(200);
      expect(await getResponse.json().then((body) => body.value)).toBe(NEW_VALUE);
    });

    it("should remove the new value & get null", async () => {
      const { proof } = await prover.prove(SECRET, NEW_VALUE, null);
      const removeResponse = await postData(url, {
        route: "REMOVE",
        data: {
          key: KEY,
          proof: proof,
        },
      });
      expect(removeResponse.status).toBe(200);

      const getResponse = await getKey(url, KEY);
      expect(getResponse.status).toBe(200);
      expect(await getResponse.json().then((body) => body.value)).toBe(null);
    });
  });

  describe("batch gets and puts", () => {
    const LENGTH = 10;
    const KEY_VALUES = Array.from({ length: LENGTH }, () =>
      randomKeyValue({
        numVals: 768,
      })
    );

    it("should put many values", async () => {
      const putResponse = await postData(url, {
        route: "PUT_MANY",
        data: {
          keys: KEY_VALUES.map((kv) => kv.key),
          values: KEY_VALUES.map((kv) => kv.value),
        },
      });
      expect(putResponse.status).toBe(200);
    });

    it("should get many values", async () => {
      const getManyResponse = await postData(url, {
        route: "GET_MANY",
        data: {
          keys: KEY_VALUES.map((kv) => kv.key),
        },
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
      const refreshResponse = await postData(url, {
        route: "REFRESH",
        data: {},
      });
      expect(refreshResponse.status).toBe(200);
    });

    it("should do a raw GET", async () => {
      const kv = KEY_VALUES[0];

      const getRawResponse = await postData(url, {
        route: "GET_RAW",
        data: {
          key: kv.key,
        },
      });
      expect(getRawResponse.status).toBe(200);
      const body = await getRawResponse.json();
      const result = body.value as (typeof kv)["value"];
      expect(result.metadata.text).toBe(kv.value.metadata.text);
      expect(result.f).toBe(kv.value.f);
    });

    it("should do a raw GET many", async () => {
      const getManyRawResponse = await postData(url, {
        route: "GET_MANY_RAW",
        data: {
          keys: KEY_VALUES.map((kv) => kv.key),
        },
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
      const kv = randomKeyValue();
      const putResponse = await postData(url, {
        route: "PUT",
        data: {
          key: kv.key,
          value: kv.value,
        },
      });
      expect(putResponse.status).toBe(200);

      // we expect only 1 new key to be added via REFRESH
      const refreshResponse = await postData(url, {
        route: "REFRESH",
        data: {},
      });
      expect(refreshResponse.status).toBe(200);
      expect(await refreshResponse.text()).toBe("1");
    });

    it("should refresh with 0 keys when no additions are made", async () => {
      // we expect 0 keys as nothing has changed since the last refresh
      const refreshResponse = await postData(url, {
        route: "REFRESH",
        data: {},
      });
      expect(refreshResponse.status).toBe(200);
      expect(await refreshResponse.text()).toBe("0");
    });

    it("should clear all keys", async () => {
      // we expect 0 keys as nothing has changed since the last refresh
      const clearResponse = await postData(url, {
        route: "CLEAR",
        data: {},
      });
      expect(clearResponse.status).toBe(200);

      const getManyRawResponse = await postData(url, {
        route: "GET_MANY_RAW",
        data: {
          keys: KEY_VALUES.map((kv) => kv.key),
        },
      });
      expect(getManyRawResponse.status).toBe(200);
      const body = await getManyRawResponse.json();

      (body.values as (typeof KEY_VALUES)[0]["value"][]).forEach((val) => expect(val).toBe(null));
    });
  });

  afterAll(async () => {
    console.log("waiting a bit before closing...");
    await sleep(1500);

    await shutdown(service);
    await arlocal.stop();
    await redisClient.quit();
  });
});
