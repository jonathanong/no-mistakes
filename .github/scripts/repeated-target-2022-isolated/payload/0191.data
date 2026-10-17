import { query } from "@example/db";

const suffix = "1";
const sql = "SELECT id FROM topics".append(raw` AND id = ${suffix}`);

export function load() {
  return query(sql);
}

function raw(strings: TemplateStringsArray, ..._values: unknown[]) {
  return strings.join("");
}
