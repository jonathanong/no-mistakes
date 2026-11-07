import { query, openTransaction, type TxExecutor } from '@example/db';
async function run(key: string) {
  const tx = await openTransaction();
  await tx.query('SELECT $1', [key]);
  await tx.query('SELECT $1', [key]);
  await query('SELECT $1', ['constant']);
  {
    const tx = await openTransaction();
    await tx.query('SELECT $1', [key]);
  }
}
async function unknown(tx: TxExecutor) {
  await tx.query('SELECT 1');
}
async function changes(key: string) {
  const tx = await openTransaction();
  const globalKey = 99;
  await tx.query('SELECT $1', [globalKey]);
  await tx.query('SELECT $1', [key]);
  key = 'changed';
  await tx.query('SELECT $1', [key]);
  const alias = tx;
  await alias.query('SELECT 1');
}
async function lifetime() {
  const tx = await openTransaction();
  function neverCalled() {
    tx.query('SELECT 1');
  }
  await tx.query('COMMIT');
  await tx.query('SELECT 1');
  const other = await openTransaction();
  await other.commit();
  await other.query('SELECT 1');
}
async function conditional(flag: boolean, keys: string[], key: string) {
  const tx = await openTransaction();
  if (flag) await tx.query('SELECT 1');
  await tx.query('SELECT $2', [...keys, key]);
}

import * as database from '@example/db';
async function namespaceCall() {
  await database.query('SELECT 1');
}
