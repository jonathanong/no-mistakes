import { query } from "@data-stores/psql";

function safe() {
  return "TRUSTED SQL";
}

function wrap({ safe }: { safe: () => string }) {
  return safe();
}

const sql = wrap({ safe: () => "untrusted" });

export function load() {
  return query(sql);
}
