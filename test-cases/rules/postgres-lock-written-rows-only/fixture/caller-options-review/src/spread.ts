import { write, sql, beginTransaction } from '@example/db';
async function forwarded(opts?: { query?: unknown }) {
  await write(sql`SELECT id FROM items FOR UPDATE`, opts);
}
async function run(args: [{ query?: unknown }?]) {
  using tx = beginTransaction();
  await forwarded({ query: tx });
  await forwarded(...args);
}
