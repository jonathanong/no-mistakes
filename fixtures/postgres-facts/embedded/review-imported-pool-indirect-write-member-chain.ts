import { pool } from "@example/db";

// must stay Dynamic: a member chain is not a proven Object.assign
const box = { Object };
box.Object.assign(pool, { query: () => undefined });

export function a() {
  return pool.query("SELECT id FROM users WHERE x = 1");
}
