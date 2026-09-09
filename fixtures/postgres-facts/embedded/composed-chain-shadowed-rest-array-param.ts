import { query } from "@data-stores/psql";

function safe() {
  return "SELECT 1";
}

function wrap(...[safe]: [() => string]) {
  return safe();
}

const sql = wrap(() => "DELETE FROM users");

export function load() {
  return query(sql);
}
