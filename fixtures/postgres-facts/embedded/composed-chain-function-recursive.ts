import { query } from "@data-stores/psql";

function cyclic() {
  return cyclic();
}

const sql = cyclic();

export function load() {
  return query(sql);
}
