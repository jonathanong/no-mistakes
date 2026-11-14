import type { PoolClient, Pool } from 'db-lib';
declare const pool: { connect(): Promise<PoolClient> };
declare const id: number;
export async function untyped() {
  let client;
  try {
    client = await pool.connect();
    await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
  } finally { client?.release(); }
}
export async function reassigned() {
  let client: PoolClient | undefined;
  try {
    client = await pool.connect();
    client = { query: async () => undefined, release() {} };
    await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
  } finally { client?.release(); }
}
export async function fakeInitializer() {
  let client: PoolClient | undefined = { query: async () => undefined, release() {} };
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function reassignedPool() {
  let pool: Pool | undefined;
  try {
    pool = await Promise.resolve({} as Pool);
    pool = { query: async () => undefined, end: async () => undefined };
    await pool.query('UPDATE items SET a=1 WHERE id=$1', [id]);
  } finally { await pool?.end(); }
}
