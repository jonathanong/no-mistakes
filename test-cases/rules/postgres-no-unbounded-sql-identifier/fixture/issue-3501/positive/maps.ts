import { query } from '@example/db';
import { assertIdent } from '../validators';
const RAW_TABLES = ['orders', 'refunds'];
const TABLES = RAW_TABLES.map(assertIdent);
const COUNTS = ['orders', 'refunds'].map((name) => `COUNT(${name})`).join(',');

export function locks() {
  return query(`LOCK TABLE ${TABLES.join(', ')}`);
}

export function counts() {
  return query(`SELECT ${COUNTS} FROM orders`);
}
