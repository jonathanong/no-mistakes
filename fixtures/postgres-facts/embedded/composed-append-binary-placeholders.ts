import { query } from "@example/db";

const text = sql`SELECT * FROM topics WHERE id = ${1}` + sql` AND status = ${2}`;

export function load() {
  return query(text);
}

function sql(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join("");
}
