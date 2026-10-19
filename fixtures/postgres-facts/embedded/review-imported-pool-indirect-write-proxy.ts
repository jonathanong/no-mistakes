import { pool } from "@example/db";

// must stay Dynamic: new Proxy receives the executor as its target
export function a() {
  return pool.query("SELECT id FROM users WHERE x = 1");
}

export function c() {
  const x: any = new Proxy(pool, {});
  x.query = 1;
}
