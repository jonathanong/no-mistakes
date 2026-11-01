import type { QueryExecutor } from '@example/db';
export function excluded(run: QueryExecutor) { run('SELECT 1'); }
