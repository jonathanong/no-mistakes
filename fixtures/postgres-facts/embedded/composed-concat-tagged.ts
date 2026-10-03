import { query } from "@example/db";

const sql = sql`SELECT id FROM ` + "topics";

export function load() {
  return query(sql);
}

function sql(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join("");
}
