import { query } from "@example/db";

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
