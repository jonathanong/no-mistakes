import pg, { Pool, Pool as RenamedPool } from "pg";
import * as postgres from "pg";

const config = { host: "db.example.invalid" };
const handleError = (error) => console.error(error);

export const pool = new Pool(config);
pool.on("connect", handleError);
pool.emit("error", handleError);
pool.addListener("error");

export const renamed = new RenamedPool(config);
export const fromDefault = new pg.Pool(config);
export const fromNamespace = new postgres.Pool(config);

export function createPool() {
  return new Pool(config);
}

export function createChainedPool() {
  return new Pool(config).on("connect", handleError);
}

let shared = new Pool(config);
shared = createChainedPool();
shared.on("error", handleError);

unresolvedPool = new Pool(config);
unresolvedPool.on("error", handleError);
