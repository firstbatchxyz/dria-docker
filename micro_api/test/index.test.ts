import http from "http";
import { serve } from "micro";
import { expect } from "chai";
import listen from "test-listen";
import { StatusCodes } from "http-status-codes";
import ArLocal from "arlocal";
import { ArWallet, LoggerFactory } from "warp-contracts";

import { Redis } from "ioredis";
import { SDK } from "hollowdb";
import { Prover, computeKey } from "hollowdb-prover";

import makeServer from "../src/server";
import config from "../src/config";
import { createCaches } from "../src/cache";
import { makeLocalWarp } from "../src/util";

import { deploy, postData, getKey, sleep, shutdown, randomKeyValue } from "./util";

describe("crud operations", () => {
  let arlocal: ArLocal;
  let service: http.Server;
  let redisClient: Redis;
  let url: string;
  let prover: Prover;
  const ARWEAVE_PORT = 3169;

  const VALUE = BigInt(Math.floor(Math.random() * 9_999_999)).toString();
  const NEW_VALUE = BigInt(Math.floor(Math.random() * 9_999_999)).toString();
  const SECRET = BigInt(Math.floor(Math.random() * 9_999_999));
  const KEY = computeKey(SECRET);

  before(async () => {
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
    prover = new Prover(__dirname + "/circuit/hollow-authz.wasm", __dirname + "/circuit/prover_key.zkey", "plonk");
    caches = createCaches(contractTxId, redisClient);
    warp = makeLocalWarp(ARWEAVE_PORT, caches);
    const hollowdb = new SDK(owner, contractTxId, warp);
    const server = makeServer(hollowdb, contractTxId);
    service = new http.Server(serve(server));
    url = await listen(service);
    console.log("micro listening at", url);
    LoggerFactory.INST.logLevel("error");

    // dont care about logs for the test
    // LoggerFactory.INST.logLevel("none");

    // wait a bit due to state syncing
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
      expect(putResponse.status).to.eq(StatusCodes.OK);

      const getResponse = await getKey(url, KEY);
      expect(getResponse.status).to.eq(StatusCodes.OK);
      expect(await getResponse.json().then((body) => body.value)).to.eq(VALUE);
    });

    it("should NOT put to an existing key", async () => {
      const putResponse = await postData(url, {
        route: "PUT",
        data: {
          key: KEY,
          value: VALUE,
        },
      });
      expect(putResponse.status).to.eq(StatusCodes.BAD_REQUEST);
      expect(await putResponse.text()).to.eq("Contract Error [put]: Key already exists.");
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
      expect(putResponse.status).to.eq(StatusCodes.BAD_REQUEST);
      expect(await putResponse.text()).to.eq("Contract Error [update]: Invalid proof.");
    });

    it("should update & get the new value", async () => {
      const { proof } = await prover.prove(SECRET, VALUE, NEW_VALUE);
      const updateResponse = await postData(url, {
        route: "UPDATE",
        data: {
          key: KEY,
          value: NEW_VALUE,
          proof: proof,
        },
      });
      expect(updateResponse.status).to.eq(StatusCodes.OK);

      const getResponse = await getKey(url, KEY);
      expect(getResponse.status).to.eq(StatusCodes.OK);
      expect(await getResponse.json().then((body) => body.value)).to.eq(NEW_VALUE);
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
      expect(removeResponse.status).to.eq(StatusCodes.OK);

      const getResponse = await getKey(url, KEY);
      expect(getResponse.status).to.eq(StatusCodes.OK);
      expect(await getResponse.json().then((body) => body.value)).to.eq(null);
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
      expect(putResponse.status).to.eq(StatusCodes.OK);
    });

    it("should get many values", async () => {
      const getManyResponse = await postData(url, {
        route: "GET_MANY",
        data: {
          keys: KEY_VALUES.map((kv) => kv.key),
        },
      });
      expect(getManyResponse.status).to.eq(StatusCodes.OK);
      const body = await getManyResponse.json();
      for (let i = 0; i < KEY_VALUES.length; i++) {
        const expected = KEY_VALUES[i].value;
        const result = body.values[i] as (typeof KEY_VALUES)[0]["value"];
        expect(result.metadata.text).to.eq(expected.metadata.text);
        expect(result.f).to.eq(expected.f);
      }
    });

    it("should refresh the cache for raw GET operations", async () => {
      const refreshResponse = await postData(url, {
        route: "REFRESH",
        data: {},
      });
      expect(refreshResponse.status).to.eq(StatusCodes.OK);
    });

    it("should do a raw GET", async () => {
      const kv = KEY_VALUES[0];
      const getRawResponse = await postData(url, {
        route: "GET_RAW",
        data: {
          key: kv.key,
        },
      });
      expect(getRawResponse.status).to.eq(StatusCodes.OK);
      const body = await getRawResponse.json();
      const result = body.value as (typeof kv)["value"];
      expect(result.metadata.text).to.eq(kv.value.metadata.text);
      expect(result.f).to.eq(kv.value.f);
    });

    it("should do a raw GET many", async () => {
      const getManyRawResponse = await postData(url, {
        route: "GET_MANY_RAW",
        data: {
          keys: KEY_VALUES.map((kv) => kv.key),
        },
      });
      expect(getManyRawResponse.status).to.eq(StatusCodes.OK);
      const body = await getManyRawResponse.json();
      for (let i = 0; i < KEY_VALUES.length; i++) {
        const expected = KEY_VALUES[i].value;
        const result = body.values[i] as (typeof KEY_VALUES)[0]["value"];
        expect(result.metadata.text).to.eq(expected.metadata.text);
        expect(result.f).to.eq(expected.f);
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
      expect(putResponse.status).to.eq(StatusCodes.OK);

      // we expect only 1 new key to be added via REFRESH
      const refreshResponse = await postData(url, {
        route: "REFRESH",
        data: {},
      });
      expect(refreshResponse.status).to.eq(StatusCodes.OK);
      expect(await refreshResponse.text()).to.eq("1");
    });

    it("should refresh with 0 keys when no additions are made", async () => {
      // we expect 0 keys as nothing has changed since the last refresh
      const refreshResponse = await postData(url, {
        route: "REFRESH",
        data: {},
      });
      expect(refreshResponse.status).to.eq(StatusCodes.OK);
      expect(await refreshResponse.text()).to.eq("0");
    });

    it("should clear all keys", async () => {
      // we expect 0 keys as nothing has changed since the last refresh
      const clearResponse = await postData(url, {
        route: "CLEAR",
        data: {},
      });
      expect(clearResponse.status).to.eq(StatusCodes.OK);

      const getManyRawResponse = await postData(url, {
        route: "GET_MANY_RAW",
        data: {
          keys: KEY_VALUES.map((kv) => kv.key),
        },
      });
      expect(getManyRawResponse.status).to.eq(StatusCodes.OK);
      const body = await getManyRawResponse.json();
      (body.values as (typeof KEY_VALUES)[0]["value"][]).forEach((val) => expect(val).to.eq(null));
    });
  });

  after(async () => {
    console.log("waiting a bit before closing...");
    await sleep(1500);

    await shutdown(service);
    await arlocal.stop();
    await redisClient.quit();
  });
});
