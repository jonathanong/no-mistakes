import { pool } from "@example/db";

// Storing the import on a holder and writing holder.pool.query must stay Dynamic.
export function a() {
  return pool.query("SELECT id FROM users WHERE x = 1");
}

export function b() {
  const holder = { pool };
  holder.pool.query = () => undefined;
}
