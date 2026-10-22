import type { Executor, PoolClient } from '@example/db';
export function run(declared: Executor, missing: PoolClient) {
  declared.query('/* declared */ SELECT 1');
  // Even an annotated literal must report the missing configuration, not execute.
  missing.query('/* omitted */ SELECT 2');
  // no-mistakes-disable-next-line postgres-require-query-annotation
  missing.query('/* next-line suppression */ SELECT 3');
  missing.query('/* line suppression */ SELECT 4'); // no-mistakes-disable-line postgres-require-query-annotation
}
