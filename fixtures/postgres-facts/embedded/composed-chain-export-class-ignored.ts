import { query } from "@data-stores/psql";

export class Unrelated {}

function buildSql() {
  return "SELECT id FROM topics".append(" WHERE id = 1");
}

const sql = buildSql();

export function load() {
  return query(sql);
}
