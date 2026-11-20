import { sql, type SqlStatement } from './sql.mjs';
type Rows = { rows: unknown[] };
type PageQuery = (statement: SqlStatement) => Promise<Rows>;
export async function queryIdPage(limit: number, selectWhere: SqlStatement, query: PageQuery) {
  selectWhere.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  const { rows } = await query(selectWhere);
  return rows;
}
export async function queryIdPagePlain(limit: number, selectWhere: SqlStatement, query: PageQuery) {
  selectWhere.append(sql` ORDER BY id LIMIT ${limit}`);
  const { rows } = await query(selectWhere);
  return rows;
}
export async function queryIdPageReturn(limit: number, selectWhere: SqlStatement, query: PageQuery) {
  selectWhere.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  return query(selectWhere);
}
export async function queryIdPageBranch(limit: number, selectWhere: SqlStatement, afterId: string | undefined, query: PageQuery) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  selectWhere.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  const { rows } = await query(selectWhere);
  return rows;
}
