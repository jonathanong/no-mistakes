import { query } from "@data-stores/psql";

const suffix = "1";
const sql = "SELECT id FROM topics".append(raw` AND id = ${suffix}`);

export function load() {
  return query(sql);
}

function raw(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join("");
}
