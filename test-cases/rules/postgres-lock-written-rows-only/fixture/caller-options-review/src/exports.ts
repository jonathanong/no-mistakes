import { write, sql, beginTransaction } from '@example/db';
async function exported(opts?: { query?: unknown }) {
  await write(sql`SELECT id FROM items FOR UPDATE`, opts);
}
export { exported };
export const metadata = 1;
async function run() {
  using tx = beginTransaction();
  await exported({ query: tx });
}
