import { write, beginTransaction } from '@example/db';
async function stringLock(id: string, opts: unknown) {
  await write('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id], opts);
}
async function templateLock(id: string, opts: unknown) {
  await write(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id], opts);
}
async function written(id: string) {
  const tx = await beginTransaction();
  await stringLock(id, { query: tx });
  await templateLock(id, { query: tx });
  await write('UPDATE items SET active = true WHERE id = $1', [id], { query: tx });
}
async function mismatched(id: string, other: string) {
  const tx = await beginTransaction();
  await stringLock(id, { query: tx });
  await templateLock(id, { query: tx });
  await write('UPDATE items SET active = true WHERE id = $1', [other], { query: tx });
}
