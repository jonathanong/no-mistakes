import { pool } from "@example/db";
import { Patcher } from "./patcher";

// An imported constructor is not a same-file write, so the query stays exact.
export function a() {
  return pool.query("SELECT id FROM users WHERE x = 1");
}

export function b() {
  new Patcher(pool);
}
