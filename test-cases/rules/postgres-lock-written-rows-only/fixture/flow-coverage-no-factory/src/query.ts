import { type TransactionQuery } from '@example/db';
async function exitsWithoutFactory(q: TransactionQuery, flag: boolean) {
  await q.query('SELECT id FROM items WHERE id = 1 FOR UPDATE');
  if (flag) return;
  await q.query('UPDATE items SET active = true WHERE id = 1');
}
async function joinsWithoutFactory(q: TransactionQuery, flag: boolean) {
  await q.query('SELECT id FROM items WHERE id = 1 FOR UPDATE');
  if (flag) {
    const branch = true;
  } else {
    const branch = false;
  }
  await q.query('UPDATE items SET active = true WHERE id = 1');
}
async function continueWithoutFactory(q: TransactionQuery, flags: boolean[]) {
  for (const flag of flags) {
    if (flag) continue;
    await q.query('SELECT id FROM items WHERE id = 1 FOR UPDATE');
  }
}
