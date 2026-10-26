import { enqueue } from "@app/jobs";
import { query } from "pg";

export async function awaited() {
  await enqueue();
}

export async function expectAdoption() {
  await expect(enqueue()).rejects.toThrow("nope");
  const promise = enqueue();
  await expect(promise).resolves.toBeUndefined();
}

export function chained() {
  enqueue().then(() => undefined).catch(() => undefined);
}

export function returnedChain() {
  return enqueue().then(() => undefined).catch(() => undefined);
}

async function adopt(callback: () => Promise<unknown>) {
  return await callback();
}

export const inline = adopt(() => enqueue());

const named = () => enqueue();
export const namedRun = adopt(named);

export async function cachedConsumer(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  if (!cache.has("k")) cache.set("k", callback());
  return await cache.get("k");
}

const accepted = () => enqueue();
export const cached = cachedConsumer(accepted);

class SqliteRunner {
  async query(_sql: string) {
    return [];
  }
  async commit(rows: { query: string }[]) {
    await this.query("BEGIN TRANSACTION");
    await this.query("COMMIT");
    await Promise.all(rows.map((row) => this.query(row.query)));
  }
}

export async function pgStatements() {
  await query("START TRANSACTION");
  await query("COMMIT");
}

class PgRunner {
  async query(_sql: string) {
    return [];
  }
  async commit() {
    await this.query("START TRANSACTION");
    await this.query("COMMIT");
  }
}
