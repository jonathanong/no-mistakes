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
  await read(`/* append helper */ WITH RECURSIVE ${statement.text}`, statement.values);
  return read(`/* repeated append helper */ ${statement.text}`, statement.values);
}

function passThrough(base: SQLStatement): SQLStatement {
  return base;
}

export async function returnedParameter(id: string) {
  const statement = passThrough(sql`${id}`);
  return read(`/* returned parameter */ ${statement.text}`, statement.values);
}

function appendOnParameter(base: SQLStatement): SQLStatement {
  return base.append(sql` WHERE id = ${1}`);
}

export async function parameterAppend(id: string) {
  const statement = appendOnParameter(sql`${id}`);
  return read(`/* parameter append */ ${statement.text}`, statement.values);
}

function fixedAppend(): SQLStatement {
  return sql``.append(sql` WHERE id = ${1}`);
}

export async function fixedAppendHelper() {
  const statement = fixedAppend();
  return read(`/* fixed append */ ${statement.text}`, statement.values);
}
