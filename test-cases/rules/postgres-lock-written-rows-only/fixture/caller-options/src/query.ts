import { write, sql, beginTransaction, type Client } from '@example/db';
async function optional(id: string, opts?: { query?: Client }) {
  await write(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`, opts);
}
async function guarded(id: string, forUpdate: boolean, opts?: { query?: Client }) {
  await write(forUpdate ? sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE` : sql`SELECT id FROM items WHERE id = ${id}`, opts);
}
async function run(id: string) {
  using tx = beginTransaction();
  await write(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`, { query: tx });
  await optional(id, { query: tx });
  await guarded(id, true, { query: tx });
  await guarded(id, false);
  await optional(id);
  await optional(id, { query: write });
  await guarded(id, true);
  await guarded(id, true, { query: write });
  await write(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`, { query: write });
  await write(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`);
}
async function callbackRun(id: string) {
  await beginTransaction(async (transaction: Client) => {
    await write(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`, { query: transaction });
    await optional(id, { query: transaction });
    await guarded(id, true, { query: transaction });
  });
  await ordinary(async (transaction: Client) => {
    await write(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`, { query: transaction });
  });
}
declare function ordinary(callback: (client: Client) => Promise<void>): Promise<void>;
