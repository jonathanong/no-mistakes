import { write, sql, beginTransaction } from '@example/db';
async function lock(id: string, opts?: { query?: unknown }) {
  await write(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`, opts);
}
async function written(id: string) {
  const tx = await beginTransaction();
  await lock(id, { query: tx });
  await write(`UPDATE items SET active = true WHERE id = $1`, [id], { query: tx });
}
async function mismatched(id: string, other: string) {
  const tx = await beginTransaction();
  await lock(id, { query: tx });
  await write(`UPDATE items SET active = true WHERE id = $1`, [other], { query: tx });
}
