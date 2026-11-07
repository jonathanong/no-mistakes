import { query, openTransaction, type TxExecutor } from '@example/db';
async function resources(key: string, options: { query?: TxExecutor }) {
  using tx = openTransaction();
  await tx.query('SELECT $1', [key]);
  await query('SELECT $1', [key], { query: tx });
  await query('SELECT $1', [key], { query: tx.query });
  {
    await using tx = await openTransaction();
    await tx.query('SELECT $1', [key]);
    await query('SELECT $1', [key], { query: tx });
  }
  await tx.query('SELECT $1', [key]);
  await query('SELECT 1', [], { query: options?.query });
  await query('SELECT 1', [], { query: missing });
  await query('SELECT 1', [], { ...options });
  await query('SELECT 1', [], { ['query']: tx });
  await query('SELECT 1', [], { query: tx, query: tx });
  await query('SELECT 1', [], options);
  await query('SELECT 1');
}
async function sibling() {
  await tx.query('SELECT 1');
}
