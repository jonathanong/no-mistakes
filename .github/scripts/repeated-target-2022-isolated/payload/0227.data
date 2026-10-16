import { query } from "@example/db";

function safe() {
  return "TRUSTED SQL";
}

export function outer({ safe }: { safe: () => string }) {
  const sql = safe();
  return query(sql);
}
