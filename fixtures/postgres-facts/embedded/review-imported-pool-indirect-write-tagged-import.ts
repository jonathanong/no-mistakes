import { pool } from "@example/db";
import { tag } from "./tag";

// An imported tag is not a same-file write, so the query stays exact; do not simplify this away.
export function a() {
  return pool.query("SELECT id FROM users WHERE x = 1");
}

export function b() {
  tag`x${pool}`;
}
