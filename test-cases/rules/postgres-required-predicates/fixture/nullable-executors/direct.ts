import type { PoolClient } from 'db-lib';
declare const pool: { connect(): Promise<PoolClient> };
declare const id: number;
export async function run() {
  let client: PoolClient | undefined;
  try {
    client = await pool.connect();
    await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
    await client.query('UPDATE items SET a=2 WHERE id=$1', [id]);
    await client.query('UPDATE items SET a=3 WHERE id=$1 AND account_id=$2', [id, id]);
  } finally { client?.release(); }
}
