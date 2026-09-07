import { query } from "@data-stores/psql";

let sql;

export function load() {
  return query(sql);
}
