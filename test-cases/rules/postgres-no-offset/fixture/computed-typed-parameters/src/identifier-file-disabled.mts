// no-mistakes-disable-file postgres-no-unbounded-sql-identifier
import type { QueryOptions } from '@example/db';
declare const key: string;
export function suppressed({ [key]: query }: QueryOptions) {
  query('SELECT 1 OFFSET 1');
}
