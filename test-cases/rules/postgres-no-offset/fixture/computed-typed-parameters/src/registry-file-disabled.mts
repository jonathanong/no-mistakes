// no-mistakes-disable-file postgres-no-mutable-sql-registry
import type { QueryOptions } from '@example/db';
declare const key: string;
export function suppressed({ [key]: query }: QueryOptions) {
  query('SELECT 1 OFFSET 1');
}
