import { query } from "@data-stores/psql";

const sql = sql`SELECT id FROM ` + "topics";

export function load() {
  return query(sql);
}

function sql(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join("");
}
