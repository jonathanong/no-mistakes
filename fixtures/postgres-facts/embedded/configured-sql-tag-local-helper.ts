import { query } from "@example/db";

const sql = (strings: TemplateStringsArray, ...values: unknown[]) =>
  strings.join("") + values.join("");

export function load(filter: string) {
  return query(sql`SELECT * FROM t WHERE ${filter}`);
}
