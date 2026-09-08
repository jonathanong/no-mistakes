import { query } from "@data-stores/psql";

function buildSql() {
  return sql`SELECT * FROM topics WHERE id = ${1}` + sql` AND status = ${2}`;
}

const text = buildSql();

export function load() {
  return query(text);
}

function sql(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join("");
}
