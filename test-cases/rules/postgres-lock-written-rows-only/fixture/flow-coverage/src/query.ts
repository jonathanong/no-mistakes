import { beginTransaction } from '@example/db';
async function nestedClosed(flag: boolean, chooseCommit: boolean, id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (flag)
    if (chooseCommit) {
      await tx.commit();
      return;
    } else {
      await tx.rollback();
      return;
    }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function unsafeNested(flag: boolean, chooseCommit: boolean, id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (flag)
    if (chooseCommit) {
      await tx.commit();
      return;
    } else {
      return;
    }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function constantTrue(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (true) {
    await tx.commit();
  } else {
    await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
  }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function constantFalse(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (false) {
    await tx.commit();
    return;
  }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function terminatingElse(flag: boolean, id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (flag) {
    const selected = true;
  } else {
    await tx.rollback();
    return;
  }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function alreadyCommitted(flag: boolean, id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  await tx.commit();
  if (flag) return;
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function conditionallyCommitted(flag: boolean, id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (flag) await tx.commit();
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
