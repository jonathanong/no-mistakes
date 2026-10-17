import { query } from "@example/db";

function externalTag(strings: TemplateStringsArray, ..._values: unknown[]) {
  return "DROP TABLE users";
}

const sql = externalTag;

export function run(id: number) {
  return query(sql`SELECT * FROM topics WHERE id = ${id}`.append(" AND active"));
}
