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
declare function pick(statement: SqlStatement): string;
type Reader = (s: SqlStatement) => Promise<Rows>;
export async function pageDefault(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const { rows = mutate(selectWhere) } = await query(selectWhere);
  return rows;
}
// The loop keeps this helper unsupported, so the append proof re-lowers the defaulted call.
export async function pageLoopDefault(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  while (afterId) break;
  const { rows = mutate(selectWhere) } = await query(selectWhere);
  return rows;
}
export async function pageComputed(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const { [pick(selectWhere)]: rows } = await query(selectWhere);
  return rows;
}
export async function pageAfter(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  await query(selectWhere);
  selectWhere.append(sql` LIMIT 1`);
}
export async function pageMutate(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const result = await query(selectWhere);
  const touched = mutate(selectWhere);
  return result;
}
export async function plainDestructure(statement: SqlStatement, run: Reader) {
  const { rows } = await run(statement);
  return rows;
}
export async function pageArray(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  await query(selectWhere);
  return [mutate(selectWhere)];
}
export async function pageObject(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  await query(selectWhere);
  const wrapped = { y: mutate(selectWhere) };
  return wrapped;
}
export async function pageSilentDefault(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const { rows = mutate() } = await query(selectWhere);
  return rows;
}
export async function pageSilentComputed(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const { [pick()]: rows } = await query(selectWhere);
  return rows;
}
export const pageArrowSilent = async (selectWhere: SqlStatement, afterId: string | undefined, query: Reader) => {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const { rows = mutate() } = await query(selectWhere);
  return rows;
};
