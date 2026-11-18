import sql from 'sql-template-strings';
import { read } from './db.mjs';
import { nativePage } from './native-pager.mjs';
import type { SqlStatement } from './sql.mjs';
export const native = (after?: string) => nativePage(sql`/* native */ SELECT id FROM widgets`, after, statement => read(statement));
async function localPage<T>(statement: SqlStatement, after: string | undefined, run: (s: SqlStatement) => Promise<T>) {
  if (after) statement.append(sql` AND id > ${after}`);
  statement.append(sql` ORDER BY id LIMIT 51`);
  return await run(statement);
}
export const localNative = (after?: string) => localPage(sql`/* localNative */ SELECT id FROM widgets`, after, statement => read(statement));
export const nativeMissing = () => nativePage(sql`SELECT id FROM widgets`, undefined, statement => read(statement));
export const nativeOpaque = (value: SqlStatement) => nativePage(value, undefined, statement => read(statement));
