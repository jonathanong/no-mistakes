import { query } from "@data-stores/psql";

const text = sql`SELECT * FROM topics WHERE id = ${1}`;
text.append(sql` AND status = ${2}`);

export function load() {
  return query(text);
}

function sql(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join("");
}
