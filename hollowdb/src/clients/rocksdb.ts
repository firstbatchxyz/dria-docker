import Levelup from "levelup";
import Rocksdb from "rocksdb";
import { toValueKey } from "../utilities/download";
import { existsSync, mkdirSync } from "fs";

export class RocksdbClient<V = any> {
  db: ReturnType<typeof Levelup>;
  contractTxId: string;

  constructor(path: string, contractTxId: string) {
    if (!existsSync(path)) {
      mkdirSync(path, { recursive: true });
    }

    this.db = Levelup(Rocksdb(path));
    this.contractTxId = contractTxId;
  }

  async close() {
    if (!this.db.isClosed()) {
      await this.db.close();
    }
  }

  async open() {
    if (!this.db.isOpen()) {
      await this.db.open();
    }
  }

  async get(key: string) {
    const value = await this.db.get(toValueKey(this.contractTxId, key));
    return this.tryParse(value);
  }

  async getMany(keys: string[]) {
    const values = await this.db.getMany(keys.map((k) => toValueKey(this.contractTxId, k)));
    return values.map((v) => this.tryParse(v));
  }

  async set(key: string, value: string) {
    await this.db.put(toValueKey(this.contractTxId, key), value);
  }

  async setMany(pairs: [string, string][]) {
    await this.db.batch(
      pairs.map(([key, value]) => ({
        type: "put",
        key: toValueKey(this.contractTxId, key),
        value: value,
      }))
    );
  }

  async remove(key: string) {
    await this.db.del(toValueKey(this.contractTxId, key));
  }

  async removeMany(keys: string[]) {
    await this.db.batch(keys.map((key) => ({ type: "del", key: toValueKey(this.contractTxId, key) })));
  }

  /**
   * Given a value, tries to `JSON.parse` it and if parsing fails
   * it will return the value as-is.
   *
   * This is particularly useful when there are some stringified values,
   * and some other non-stringified strings together.
   *
   * @param value a stringified or null value
   * @template V type of the expected value
   * @returns parsed value or `null`
   */
  private tryParse(value: Rocksdb.Bytes | null): V | null {
    let result = null;

    if (value) {
      try {
        result = JSON.parse(value.toString());
      } catch (err) {
        // FIXME: return null here?
        result = value;
      }
    }

    return result as V;
  }
}
