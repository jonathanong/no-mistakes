import { pool } from "@example/db";

// must stay Dynamic: the same-file constructor writes .query
class Patcher {
  constructor(x: { query: unknown }) {
    x.query = () => undefined;
  }
}

export function a() {
  return pool.query("SELECT id FROM users WHERE x = 1");
}

export function b() {
  new Patcher(pool);
}
