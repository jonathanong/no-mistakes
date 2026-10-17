import { query } from "@example/db";

function cyclic() {
  return cyclic();
}

const sql = cyclic();

export function load() {
  return query(sql);
}
