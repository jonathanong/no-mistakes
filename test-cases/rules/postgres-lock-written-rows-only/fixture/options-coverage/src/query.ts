import { query, beginTransaction } from '@example/db';
async function rejectedOverrides(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  const options = { query: execute };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options, options);
  await query('UPDATE items SET active = true WHERE id = $1', [id], options, { query: tx });
  await query('UPDATE items SET active = true WHERE id = $1', [id], { [id]: tx });
  await query('UPDATE items SET active = true WHERE id = $1', [id], { ...options });
  await query('UPDATE items SET active = true WHERE id = $1', [id], { query: other().query });
  await query('UPDATE items SET active = true WHERE id = $1', [id], false);
  const unknown = { query: other().query };
  await query('UPDATE items SET active = true WHERE id = $1', [id], unknown);
}
async function lifecycleAndShadow(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  const alias = tx;
  const options = { query: execute };
  {
    const tx = {};
    await alias.commit();
  }
  await alias.rollback();
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function unknownReceivers(id: string, holder: { current: { query: unknown } }) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  const options = { query: execute };
  holder.current.query = query;
  external.query = query;
  unknown.commit();
  let mutable = tx;
  mutable.commit();
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function declarations(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const { query: execute } = tx;
  const options = (options = { query: tx.query });
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function compliant(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  const options = { query: execute };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
