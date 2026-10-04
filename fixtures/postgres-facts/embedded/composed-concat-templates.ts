import { query } from "@example/db";

const sql = `SELECT id FROM ` + `topics`;

export function load() {
  return query(sql);
}
