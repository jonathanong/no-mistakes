import { query, sql } from '@example/db';
export function capturedDefault(table: string, change = () => { table = 'records; DROP TABLE records; --'; }) {
  const allowed = new Set(['safe_one', 'safe_two']);
  if (!allowed.has(table)) throw new Error('invalid table');
  const changed = change(), statement = sql`/* defaultCapture */ SELECT 1 FROM `.append(table);
  return query(statement);
}
