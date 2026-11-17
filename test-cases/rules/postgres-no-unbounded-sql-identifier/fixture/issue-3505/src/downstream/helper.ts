import sql, { type SQLStatement } from "sql-template-strings";
import { read } from "db";

function extend(base: SQLStatement): SQLStatement {
  return sql`SELECT * FROM items WHERE id = ${base}`;
}

export async function helper(id: string) {
  const statement = extend(sql`${id}`);
  return read(`/* tag */ WITH RECURSIVE ${statement.text}`, statement.values);
}

function extendByAppend(base: SQLStatement): SQLStatement {
  return sql``.append(base);
}

export async function helperAppend(id: string) {
  const statement = extendByAppend(sql`${id}`);
  return read(`/* append helper */ WITH RECURSIVE ${statement.text}`, statement.values);
}
