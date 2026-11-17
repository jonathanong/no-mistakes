import { write, sql, beginTransaction } from '@example/db';
async function conflicting(opts?: { query?: unknown }) {
  await write(sql`SELECT id FROM items FOR UPDATE`, opts, { query: write });
}
async function run() {
  using tx = beginTransaction();
  await conflicting({ query: tx });
}
