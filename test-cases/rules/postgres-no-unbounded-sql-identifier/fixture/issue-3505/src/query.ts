import sql from "sql-template-strings";
import { read, write } from "db";
import * as statements from "./namespace";

const insert = (id: string, body: string) =>
  sql`INSERT INTO t (id, body) VALUES (${id}, ${body})`;

export async function helper(id: string, body: string) {
  await write(insert(id, body));
  const stmt = insert(id, body);
  stmt.append(sql` ON CONFLICT (id) DO UPDATE SET body = ${body}`);
  await write(stmt);
}

export async function text(a: string, b: string) {
  const q = sql`SELECT 1 WHERE a = ${a}`;
  q.append(sql` AND b = ${b}`);
  await read(`/* tag */ ${q.text}`, q.values);
}

function literalStatement() {
  return sql`SELECT * FROM items`;
}

export async function noBindValuesNeeded() {
  const local = sql`SELECT * FROM items`;
  await read(`/* literal */ ${local.text}`);
  const returned = literalStatement();
  return read(`/* literal helper */ ${returned.text}`);
}

export async function repeatedNoBindFragment() {
  const fragment = sql`id`;
  const statement = sql``.append(fragment).append(fragment);
  return read(`/* repeated */ SELECT ${statement.text}`);
}

export async function appendAfterFirstUse(id: string) {
  const statement = sql`SELECT * FROM items`;
  await read(`/* before append */ ${statement.text}`);
  statement.append(sql` WHERE id = ${id}`);
  return read(`/* after append */ ${statement.text}`, statement.values);
}

export async function matchedNamespaceValues() {
  return read(`/* matched namespace */ ${statements.namespaceStatement.text}`,
    statements.namespaceStatement.values);
}

export async function multiHoleNoBinds() {
  const q = sql`SELECT * FROM items`;
  const suffix = sql` ORDER BY id`;
  return read(`/* multi literal */ ${q.text}${suffix.text}`);
}
