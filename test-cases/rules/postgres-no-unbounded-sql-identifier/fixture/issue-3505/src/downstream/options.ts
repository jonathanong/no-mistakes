import { read } from "db";
import { importedWithOptions } from "./producer";
import sql, { type SQLStatement } from "sql-template-strings";

function extend(sourceQuery: SQLStatement, options: { limit: number; marker: string }): SQLStatement {
  return sql`SELECT * FROM items WHERE id = ${sourceQuery} LIMIT ${options.limit}`;
}

export async function options(limit: number, marker: string) {
  const options = { limit, marker };
  const sourceQuery = sql`${marker}`;
  const statement = extend(sourceQuery, options);
  const q = importedWithOptions(options);
  await read(`/* options */ ${statement.text}`, statement.values, { rowMode: "array", limit });
  return read(q.text, q.values, options);
}
