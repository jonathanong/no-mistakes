import type { Pool, PoolClient } from 'db-lib';
declare const pool: { connect(): Promise<PoolClient> };
declare function openPool(): Promise<Pool>;
declare const id: number;
export async function runClient() {
  let client: PoolClient | undefined;
  try {
    client = await pool.connect();
    await client?.query('UPDATE items SET a=1 WHERE id=$1', [id]);
    await client?.query('UPDATE items SET a=3 WHERE id=$1 AND account_id=$2', [id, id]);
  } finally { client?.release(); }
}
export async function runPool() {
  let opened: Pool | undefined;
  try {
    opened = await openPool();
    await opened?.query('UPDATE items SET a=2 WHERE id=$1', [id]);
    await opened?.query('UPDATE items SET a=4 WHERE id=$1 AND account_id=$2', [id, id]);
  } finally { await opened?.end(); }
}
