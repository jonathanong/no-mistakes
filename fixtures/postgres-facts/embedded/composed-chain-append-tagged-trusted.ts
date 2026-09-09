import { query } from "@data-stores/psql";

const sql = "SELECT id FROM topics".append(sql` AND active`);

export function load() {
  return query(sql);
}

function sql(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join("");
}
