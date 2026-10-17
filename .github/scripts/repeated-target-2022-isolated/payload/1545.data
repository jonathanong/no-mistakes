import { query } from "@example/db";

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
