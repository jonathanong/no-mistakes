import type { PoolClient } from './delayed-try-db.mjs';
declare const pool: { connect(): Promise<PoolClient> };

export async function relative() {
  let client: PoolClient | undefined;
  try {
    client = await pool.connect();
    await client.query('/* relative_named */ SELECT 1');
    await client.query('SELECT 2');
  } finally { client?.release(); }
}
