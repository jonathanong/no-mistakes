import { query, read, write, custom, sql, beginTransaction } from '@example/db';
query('SELECT id FROM query_items FOR UPDATE');
read('SELECT id FROM read_items FOR UPDATE');
write('SELECT id FROM write_items FOR UPDATE');
custom('SELECT id FROM custom_items FOR UPDATE');
sql('SELECT id FROM tag_items FOR UPDATE');
async function scoped() {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM scoped_items FOR UPDATE');
}
