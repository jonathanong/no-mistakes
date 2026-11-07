import { query, beginTransaction } from '@example/db';
async function conditionalClose(skip: boolean, close: boolean, id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (skip) {
    if (close) await tx.commit();
    return;
  }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function bothArmsClose(skip: boolean, commit: boolean, id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (skip) {
    if (commit) await tx.commit();
    else await tx.rollback();
    return;
  }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function unrelatedClose(skip: boolean, id: string) {
  await using tx = await beginTransaction();
  await using unrelated = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (skip) {
    await unrelated.commit();
    return;
  }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function reassignedThenReturn(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    tx = null;
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function unrelatedShadowAssignment(skip: boolean, id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const alias = tx;
  if (skip) {
    {
      let tx = 1;
      tx = 2;
    }
    await alias.commit();
    return;
  }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
