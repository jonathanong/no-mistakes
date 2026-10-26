import type { QueryOptions } from '@example/db';
declare const key: string;
export function suppressed({ [key]: query }: QueryOptions) {
  // no-mistakes-disable-next-line postgres-no-mutable-sql-registry
  query('SELECT 1 OFFSET 1');
  query('SELECT 1'); // no-mistakes-disable-line postgres-no-mutable-sql-registry
}
