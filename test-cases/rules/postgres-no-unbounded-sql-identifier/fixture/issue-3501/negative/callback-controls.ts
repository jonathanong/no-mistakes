import { query } from '@example/db';
import type { assertIdent as typeOnlyValidator } from '../validators';
import { assertIdent } from '../validators';
const RAW = ['orders', 'refunds'];
const TYPE_ONLY = RAW.map(typeOnlyValidator);
const SPREAD = RAW.map(...[assertIdent]);

export function typeOnly() { return query(`LOCK TABLE ${TYPE_ONLY.join(', ')}`); }
export function spread() { return query(`LOCK TABLE ${SPREAD.join(', ')}`); }
export function mutable() {
  let callback = assertIdent;
  callback = readCallback();
  const tables = RAW.map(callback);
  return query(`LOCK TABLE ${tables.join(', ')}`);
}
