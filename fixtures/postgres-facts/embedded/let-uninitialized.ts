import { query } from "@example/db";

let sql;

export function load() {
  return query(sql);
}
