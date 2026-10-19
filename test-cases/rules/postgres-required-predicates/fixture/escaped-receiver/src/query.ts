import { pool } from "@example/db";

export function load() {
  return pool.query("SELECT id FROM users");
}

export function replace() {
  pool.query = () => undefined;
}

export function unknown(db: { query(text: string): unknown }) {
  db.query("SELECT id FROM other");
}
