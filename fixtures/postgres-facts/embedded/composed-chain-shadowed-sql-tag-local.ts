import { query } from "@data-stores/psql";

function run(sql: (strings: TemplateStringsArray, ...values: unknown[]) => string, id: number) {
  const text = sql`SELECT * FROM topics WHERE id = ${id}`;
  return query(text);
}

export function load() {
  return run((strings) => strings.join(""), 1);
}
