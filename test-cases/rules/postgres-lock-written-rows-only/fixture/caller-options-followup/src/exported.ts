import { write, sql, beginTransaction } from '@example/db';
export async function exportedLock(options: { query?: unknown }) {
  await write(sql`SELECT id FROM users WHERE id = 1 FOR UPDATE`, options);
}
export const metadata = 1;
async function run() {
  await using tx = await beginTransaction();
  await exportedLock({ query: tx });
}
