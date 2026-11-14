import type { Pool } from 'db-lib';
declare function openPool(): Promise<Pool>;
declare const id: number;
let pool: Pool | undefined;
try {
  pool = await openPool();
  await pool.query('UPDATE items SET a=1 WHERE id=$1', [id]);
  await pool.query('UPDATE items SET a=2 WHERE id=$1', [id]);
} finally { await pool?.end(); }
