import { write, sql, beginTransaction } from '@example/db';
export async function exported(opts?: { query?: unknown }) {
  await write(sql`SELECT id FROM items FOR UPDATE`, opts);
}
async function escaped(opts?: { query?: unknown }) {
  await write(sql`SELECT id FROM items FOR UPDATE`, opts);
}
async function reassigned(opts?: { query?: unknown }) {
  opts = undefined;
  await write(sql`SELECT id FROM items FOR UPDATE`, opts);
}
async function mutated(opts: { query?: unknown }) {
  opts.query = write;
  await write(sql`SELECT id FROM items FOR UPDATE`, opts);
}
async function unknownFlag(flag: boolean, opts?: { query?: unknown }) {
  await write(flag ? sql`SELECT id FROM items FOR UPDATE` : sql`SELECT id FROM items`, opts);
}
async function run(flag: boolean) {
  using tx = beginTransaction();
  await exported({ query: tx });
  await escaped({ query: tx });
  consume(escaped);
  await reassigned({ query: tx });
  await mutated({ query: tx });
  await unknownFlag(flag, { query: tx });
}
declare function consume(callback: unknown): void;
