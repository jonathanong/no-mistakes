import pg, { Pool, type Pool as InlineTypePool } from "pg";
import type { Pool as TypePool } from "pg";

const config = { host: "db.example.invalid" };
const handleError = (error: Error) => console.error(error);
const pool = new Pool(config);
const ignored = new TypePool(config);
const inlineIgnored = new InlineTypePool(config);

function register(pool: Pool) {
  pool.on("error", handleError);
}

function useDifferentConstructor(Pool: typeof InlineTypePool, pg: typeof import("pg")) {
  return [new Pool(config), new pg.Pool(config)];
}

const local = new pg.Pool(config) satisfies Pool;
(local as Pool).on("error" as const, handleError);

function createLocal() {
  const pool = new Pool(config);
  pool.on("error", handleError);
  return pool;
}
