import { query } from "@example/db";

function build(sql: (strings: TemplateStringsArray, ...values: unknown[]) => string, id: number) {
  return sql`SELECT * FROM topics WHERE id = ${id}`;
}

export function load() {
  return query(build((strings) => strings.join(""), 1));
}
