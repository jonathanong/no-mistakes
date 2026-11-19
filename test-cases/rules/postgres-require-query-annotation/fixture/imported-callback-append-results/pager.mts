import { sql, type SqlStatement } from './sql.mjs';
type Rows = { rows: unknown[] };
type Reader = (s: SqlStatement) => Promise<Rows>;
type List = (s: SqlStatement) => Promise<unknown[]>;
declare function mutate(statement: SqlStatement): boolean;
export async function pageMember(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const result = await query(selectWhere);
  return result.rows;
}
export async function pageAwaited(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const result = query(selectWhere);
  return await result;
}
export async function pageObjectRest(selectWhere: SqlStatement, afterId: string | undefined, query: Reader) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const { rows, ...meta } = await query(selectWhere);
  return rows;
}
export async function pageArrayRest(selectWhere: SqlStatement, afterId: string | undefined, query: List) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const [first, ...others] = await query(selectWhere);
  return first;
}
export async function pageArrayDefault(selectWhere: SqlStatement, afterId: string | undefined, query: List) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  const [first = mutate(selectWhere)] = await query(selectWhere);
  return first;
}
