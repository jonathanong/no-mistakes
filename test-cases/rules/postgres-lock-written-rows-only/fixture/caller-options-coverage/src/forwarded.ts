import { write, sql, beginTransaction } from '@example/db';
async function select(flag: unknown, opts?: { query?: unknown }) {
  await write(flag ? sql`SELECT id FROM items FOR UPDATE` : sql`SELECT id FROM items`, opts);
}
async function forward(flag: boolean, opts?: { query?: unknown }) {
  await select(flag, opts);
}
async function literalFlag(opts: { query?: unknown }) {
  await select('enabled', opts);
}
async function run() {
  using tx = beginTransaction();
  await forward(true, { query: tx });
  await forward(false);
  await literalFlag({ query: write });
}
