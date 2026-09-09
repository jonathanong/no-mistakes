import { query } from "@data-stores/psql";

function safe() {
  return "TRUSTED SQL";
}

function wrap(safe: () => string) {
  return safe();
}

const sql = wrap(() => "untrusted");

export function load() {
  return query(sql);
}
