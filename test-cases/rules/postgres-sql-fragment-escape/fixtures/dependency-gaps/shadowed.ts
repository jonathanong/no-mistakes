import sql from "@example/sql";
import "./shadowed-broken";

export function read(sql: (template: TemplateStringsArray) => unknown) {
  return sql`select 1`;
}
