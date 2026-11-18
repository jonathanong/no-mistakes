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
