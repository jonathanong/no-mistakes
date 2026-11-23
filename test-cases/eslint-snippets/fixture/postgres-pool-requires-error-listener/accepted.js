import pg, { Pool, Pool as RenamedPool } from "pg";
import * as postgres from "pg";

const config = { host: "db.example.invalid" };
const handleError = (error) => console.error(error);

const pool = new Pool(config);
pool.on("error", handleError);

function registerAliasListener() {
  renamed.once("error", handleError);
}
const renamed = new RenamedPool(config);
registerAliasListener();

const fromDefault = new pg.Pool(config);
fromDefault.addListener("error", handleError);

const fromNamespace = new postgres["Pool"](config);
fromNamespace[`on`](`error`, handleError);

let assigned;
assigned = new Pool(config);
assigned.on("error", handleError);

export function replacePool(pool) {
  pool = new Pool(config);
  pool.on("error", handleError);
  return pool;
}

export function createPool() {
  const local = new Pool(config);
  local.on("error", handleError);
  return local;
}

export function createChainedPool() {
  return new Pool(config).on("error", handleError);
}

export const singleClient = new pg.Client(config);
