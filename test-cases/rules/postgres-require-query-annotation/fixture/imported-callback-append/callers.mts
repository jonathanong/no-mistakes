import { read } from './db.mjs';
import { page, replaced, prepended } from './pager.mjs';
import { sql, type SqlStatement } from './sql.mjs';
export const first = (after?: string) => page(sql`/* pageWidgets */ SELECT id FROM widgets WHERE TRUE`, after, 50, statement => read(statement));
export const second = (after?: string) => page(sql`/* pageGadgets */ SELECT id FROM gadgets WHERE TRUE`, after, 50, statement => read(statement));
export const missing = () => page(sql`SELECT id FROM widgets`, undefined, 50, statement => read(statement));
export const opaque = (value: SqlStatement) => page(value, undefined, 50, statement => read(statement));
export const replacement = () => replaced(sql`/* replacement */ SELECT id FROM widgets`, statement => read(statement));
export const prepend = () => prepended(sql`/* prepend */ SELECT id FROM widgets`, statement => read(statement));
export const early = () => {
  statement.append(sql` LIMIT 10`);
  const statement = sql`/* early */ SELECT id FROM widgets`;
  return page(statement, undefined, 50, statement => read(statement));
};
async function localPage<T>(selectWhere: SqlStatement, afterId: string | undefined, limit: number, query: (s: SqlStatement) => Promise<T>) {
  if (afterId) selectWhere.append(sql` AND id > ${afterId}`);
  selectWhere.append(sql` ORDER BY id LIMIT ${limit + 1}`);
  return await query(selectWhere);
}
export const local = (after?: string) => localPage(sql`/* local */ SELECT id FROM widgets`, after, 50, statement => read(statement));
export const shared = () => {
  const statement = sql`/* shared */ SELECT id FROM widgets`;
  return page(statement, undefined, 50, value => read(value));
};
