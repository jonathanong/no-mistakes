import sql from 'sql-template-strings';
import type { SqlStatement } from './sql.mjs';
export async function nativePage<T>(statement: SqlStatement, after: string | undefined, run: (s: SqlStatement) => Promise<T>) {
  if (after) statement.append(sql` AND id > ${after}`);
  statement.append(sql` ORDER BY id LIMIT 51`);
  return await run(statement);
}
