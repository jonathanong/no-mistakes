import { query } from "@data-stores/psql";

function safe() {
  return "TRUSTED SQL";
}

export function outer({ safe }: { safe: () => string }) {
  const sql = safe();
  return query(sql);
}
