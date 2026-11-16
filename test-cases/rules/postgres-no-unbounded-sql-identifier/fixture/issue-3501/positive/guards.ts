import { query } from '@example/db';
const OK_TABLES = ['orders', 'refunds'];

export function guarded(table: string) {
  if (!OK_TABLES.includes(table)) throw new Error('bad table');
  return query(`SELECT 1 FROM ${table}`);
}
