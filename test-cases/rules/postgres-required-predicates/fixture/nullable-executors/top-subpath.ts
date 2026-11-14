import type { PoolClient } from 'db-lib/types';
declare const pool: { connect(): Promise<PoolClient> };
declare const id: number;
let client: PoolClient | undefined;
try {
  client = await pool.connect();
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
  await client.query('UPDATE items SET a=2 WHERE id=$1', [id]);
} finally { client?.release(); }
