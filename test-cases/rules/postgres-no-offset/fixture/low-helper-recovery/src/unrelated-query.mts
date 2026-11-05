import SQL from 'sql-template-strings';
import type { QueryExecutor } from '@example/db';

export function selected(query: QueryExecutor) {
  query(SQL`SELECT 1`);
}
