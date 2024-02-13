import Levelup from "levelup";
import Rocksdb from "rocksdb";

export class RocksdbClient {
  db: ReturnType<typeof Levelup>;

  constructor(path: string) {
    this.db = Levelup(Rocksdb(path));
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
    return await this.db.get(key);
  }

  async getMany(keys: string[]) {
    return await this.db.getMany(keys);
  }

  async set(key: string, value: string) {
    await this.db.put(key, value);
  }

  async setMany(pairs: [string, string][]) {
    await this.db.batch(
      pairs.map(([key, value]) => ({
        type: "put",
        key: key,
        value: value,
      }))
    );
  }

  async remove(key: string) {
    await this.db.del(key);
  }

  async removeMany(keys: string[]) {
    await this.db.batch(keys.map((key) => ({ type: "del", key })));
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
   * @returns parsed value, or `null` if it could not be parsed
   */
  static tryParse<V>(value: Rocksdb.Bytes | null): V | null {
    let result = null;

    if (value) {
      try {
        result = JSON.parse(value.toString());
      } catch (err) {
        result = value;
      }
    }

    return result;
  }
}
