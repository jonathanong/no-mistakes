import { query, beginTransaction } from '@example/db';
async function member(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function shorthand(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  const options = { query: execute };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function directShorthand(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const query = tx.query;
  const options = { query };
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function unwritten(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  await query('UPDATE other_items SET active = true WHERE id = $1', [id], options);
}
async function mutated(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  options.query = query;
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function escaped(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  change(options);
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function mutable(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  let options = { query: tx.query };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function committed(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  await tx.commit();
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
