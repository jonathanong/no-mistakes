import { query } from "@data-stores/psql";

async function buildSql() {
  return "SELECT 1";
}

const sql = buildSql();

export function load() {
  return query(sql);
}
