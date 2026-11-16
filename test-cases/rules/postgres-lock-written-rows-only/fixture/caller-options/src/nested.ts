import { write, sql, beginTransaction } from '@example/db';
async function lock(flag: boolean, opts?: { query?: unknown }) {
  await write(flag ? sql`SELECT id FROM items FOR UPDATE` : sql`SELECT id FROM items`, opts);
}
async function wrapper(opts?: { query?: unknown }) {
  await lock(true, opts);
}
async function run() {
  using tx = beginTransaction();
  await wrapper({ query: tx });
  await wrapper();
}
