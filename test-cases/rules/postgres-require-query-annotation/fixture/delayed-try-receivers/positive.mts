import type { PoolClient } from 'pg';
declare const pool: { connect(): Promise<PoolClient> };

export async function run() {
  let client: PoolClient | undefined;
  try {
    client = await pool.connect();
    await client.query('/* named_try */ SELECT 1');
    await client.query('SELECT 2');
  } finally {
    client?.release();
  }
}
