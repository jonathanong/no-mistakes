import { query, beginTransaction } from '@example/db';
async function run(id: string, options: { query?: unknown }) {
  using tx = beginTransaction();
  await query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id], { query: tx });
  await query('UPDATE items SET active = true WHERE id = $1', [id], { query: tx.query });
  {
    await using tx = await beginTransaction();
    await query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id], { query: tx });
  }
  await query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id], { query: options?.query });
  await query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id], options);
  await query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id], { query: tx, timeout: 100 });
  await query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id], ...[options]);
}
