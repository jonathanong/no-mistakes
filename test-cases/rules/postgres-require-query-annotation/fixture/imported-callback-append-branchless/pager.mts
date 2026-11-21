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
export async function queryItemsAfterCall(limit: number, selectWhere: SqlStatement, query: PageQuery) {
  const { rows } = await query(selectWhere);
  selectWhere.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  return rows;
}
export async function queryItemsAlias(limit: number, selectWhere: SqlStatement, query: PageQuery) {
  const builder = selectWhere;
  builder.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  const { rows } = await query(builder);
  return rows;
}
export async function queryItemsReassign(limit: number, selectWhere: SqlStatement, query: PageQuery) {
  let builder = selectWhere;
  builder = builder.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  const { rows } = await query(builder);
  return rows;
}
export async function queryItemsDefault(limit: number, selectWhere: SqlStatement, query: PageQuery) {
  selectWhere.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  const { rows = [] } = await query(selectWhere);
  return rows;
}
export async function queryItemsTernary(limit: number, selectWhere: SqlStatement, query: PageQuery, flag: boolean) {
  selectWhere.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  const { rows } = await query(selectWhere);
  return flag ? rows : [];
}
export async function queryItemsSpread(limit: number, selectWhere: SqlStatement, query: PageQuery) {
  selectWhere.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  const { rows } = await query(selectWhere);
  return [...rows];
}
export async function queryFlagPage(selectWhere: SqlStatement, f: string | undefined, query: PageQuery) {
  if (f) selectWhere.append(sql` AND a`);
  const { rows } = await query(selectWhere);
  return rows;
}
