import { query } from "@example/db";

const text = sql`SELECT ${1}`;
text.append(" AS sql_placeholder_1");

export function load() {
  return query(text);
}

function sql(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join("");
}
