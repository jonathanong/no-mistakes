import { query, openTransaction } from '@example/db';
async function run(id: string) {
  using tx = openTransaction();
  await tx.query('SELECT $1', [id]);
  await using other = await openTransaction();
  await other.query('SELECT $1', [id]);
  const stable = await openTransaction();
  await stable.query('SELECT $1', [id]);
  await query('SELECT $1', [id]);
}
