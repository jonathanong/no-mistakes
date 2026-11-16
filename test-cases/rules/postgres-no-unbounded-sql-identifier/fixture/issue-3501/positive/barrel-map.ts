import { query } from '@example/db';
import { assertIdent, validate as renamed } from './validator-barrel';
const RAW = ['orders', 'refunds'];
const TABLES = RAW.map(assertIdent);
const RENAMED = RAW.map(renamed);
export function barrel() { return query(`LOCK TABLE ${TABLES.join(', ')}`); }
export function alias() { return query(`LOCK TABLE ${RENAMED.join(', ')}`); }
export function directAlias(value: string) {
  const table = renamed(value);
  return query(`SELECT 1 FROM ${table}`);
}
