import { write, sql, beginTransaction } from '@example/db';
function locking(opts?: { query?: unknown }) {
  return write(sql`SELECT id FROM items FOR UPDATE`, opts);
}
async function safe() {
  using tx = beginTransaction();
  await locking({ query: tx });
}
function unsafe() {
  return locking({ query: write });
}
