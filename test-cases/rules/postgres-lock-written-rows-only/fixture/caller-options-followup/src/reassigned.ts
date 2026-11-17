import { write, sql, beginTransaction, pool } from '@example/db';
async function lockReassigned(options: { query?: unknown }) {
  await write(sql`SELECT id FROM users WHERE id = 1 FOR UPDATE`, options);
}
export async function reassigned() {
  let tx = await beginTransaction();
  tx = pool;
  await lockReassigned({ query: tx });
}
