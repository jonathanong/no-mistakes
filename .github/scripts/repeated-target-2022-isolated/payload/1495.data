import { query } from "@example/db";

export class Unrelated {}

function buildSql() {
  return "SELECT id FROM topics".append(" WHERE id = 1");
}

const sql = buildSql();

export function load() {
  return query(sql);
}
