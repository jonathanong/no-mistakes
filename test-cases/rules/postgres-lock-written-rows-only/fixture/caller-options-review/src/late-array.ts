import { write, sql, beginTransaction } from '@example/db';
async function late(opts: { query?: unknown }) {
  await write(sql`SELECT id FROM items FOR UPDATE`, opts, [mutate(opts)]);
}
function mutate(opts: { query?: unknown }) {
  opts.query = write;
  return 0;
}
async function run() {
  using tx = beginTransaction();
  await late({ query: tx });
}
