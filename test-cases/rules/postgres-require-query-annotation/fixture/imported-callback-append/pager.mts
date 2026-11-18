import { sql, type SqlStatement } from './sql.mjs';
type Run<T> = (statement: SqlStatement) => Promise<T>;
export async function page<T>(selectWhere: SqlStatement, afterId: string | undefined, limit: number, query: Run<T>) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  selectWhere.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  return await query(selectWhere);
}
export async function replaced<T>(statement: SqlStatement, query: Run<T>) {
  statement = sql`SELECT id FROM replacements`;
  return query(statement);
}
export async function prepended<T>(statement: SqlStatement, query: Run<T>) {
  return query(sql`SELECT 1; ${statement}`);
}
export async function nested<T>(statement: SqlStatement, after: string | undefined, mode: number, query: Run<T>) {
  if (mode > 0) {
    if (after) statement.append(sql` AND id > ${after}`);
  }
  return await query(statement);
}
declare function mutate(statement: SqlStatement): void;
export async function escaped<T>(statement: SqlStatement, after: string | undefined, query: Run<T>) {
  mutate(statement);
  if (after) statement.append(sql` LIMIT 10`);
  return await query(statement);
}
type Rows = { rows: unknown[] };
export async function pageRows(selectWhere: SqlStatement, afterId: string | undefined, limit: number, query: (s: SqlStatement) => Promise<Rows>) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  selectWhere.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  const { rows } = await query(selectWhere);
  return rows;
}
export async function pageBound(selectWhere: SqlStatement, afterId: string | undefined, query: Run<void>) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const result = await query(selectWhere);
  return result;
}
export async function pageStatement(selectWhere: SqlStatement, afterId: string | undefined, query: Run<void>) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  await query(selectWhere);
}
export async function otherBuilder(selectWhere: SqlStatement, other: SqlStatement, afterId: string | undefined, query: Run<void>) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const result = await query(other);
  return result;
}
