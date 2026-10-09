import sql, { type SQLStatement } from "sql-template-strings";

export function customQuery(
  strings: TemplateStringsArray,
  ..._values: unknown[]
): SQLStatement {
  return sql(strings[0]);
}
