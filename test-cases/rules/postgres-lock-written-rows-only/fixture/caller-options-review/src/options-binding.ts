import { write, sql, beginTransaction } from '@example/db';
async function optional(opts?: { query?: unknown }) {
  await write(sql`SELECT id FROM items FOR UPDATE`, opts);
}
async function run() {
  using tx = beginTransaction();
  const opts = { query: tx };
  await write(sql`SELECT id FROM items FOR UPDATE`, opts);
  await optional(opts);
}
