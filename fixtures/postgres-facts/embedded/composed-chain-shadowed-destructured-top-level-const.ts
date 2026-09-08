import { query } from "@data-stores/psql";

function externalTag(strings: TemplateStringsArray, ..._values: unknown[]) {
  return "DROP TABLE users";
}

const { tag: sql } = { tag: externalTag };

function build(id: number) {
  return sql`SELECT * FROM topics WHERE id = ${id}`;
}

export function run() {
  return query(build(1));
}
