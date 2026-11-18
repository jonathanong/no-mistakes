import { read } from './db.mjs';
import { page, prepend } from './pager.mjs';
import { sql, type SqlStatement } from './sql.mjs';
import { sql as aliasedSql } from '././sql.mjs';

export const first = (after?: string) => page(sql`/* first */ SELECT id FROM widgets WHERE true`, after, statement => read(statement));
export const second = (after?: string) => page(sql`/* second */ SELECT id FROM gadgets WHERE true`, after, statement => read(statement));

export const missing = () => page(sql`SELECT id FROM widgets WHERE true`, undefined, statement => read(statement));
export const opaque = (statement: SqlStatement) => page(statement, undefined, value => read(value));
export const prepended = () => prepend(sql`/* prepended */ SELECT id FROM widgets`, statement => read(statement));

export const unresolved = () => import('./unavailable.mjs').then(({ page }) =>
  page(sql`/* unresolved */ SELECT id FROM widgets`, undefined, (statement: SqlStatement) => read(statement)),
);

async function localPage<T>(statement: SqlStatement, after: string | undefined, run: (value: SqlStatement) => Promise<T>) {
  const tail = after ? sql` AND id > ${after}` : sql``;
  return run(sql`${statement}${tail} ORDER BY id LIMIT 50`);
}

export const local = (after?: string) => localPage(sql`/* local */ SELECT id FROM widgets WHERE true`, after, statement => read(statement));

export const alias = () => page(aliasedSql`/* alias */ SELECT id FROM widgets`, undefined, statement => read(statement));
