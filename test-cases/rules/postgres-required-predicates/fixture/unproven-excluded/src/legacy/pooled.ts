import { pool } from "@example/db";

// A configured executor call in an excluded file must not become a finding.
export function v() {
  return pool.query("SELECT id FROM users WHERE x = 2");
}
