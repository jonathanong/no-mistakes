import { beginTransaction, pool } from '@example/db';
async function lockDirect(client: { query: (sql: string) => Promise<unknown> }) {
  await client.query('SELECT id FROM users WHERE id = 1 FOR UPDATE');
}
export async function mixed() {
  await using tx = await beginTransaction();
  await lockDirect(tx);
  await lockDirect(pool);
}
export async function mixedReverse() {
  await using tx = await beginTransaction();
  await lockDirect(pool);
  await lockDirect(tx);
}
