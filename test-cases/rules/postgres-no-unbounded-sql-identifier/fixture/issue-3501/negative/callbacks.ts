import { query } from '@example/db';
import { assertIdent as wrongValidator } from '../open-data';
import { assertIdent } from '../validators';
const RAW = ['orders', 'refunds'];
const WRONG = RAW.map(wrongValidator);
const BOUND = RAW.map(assertIdent.bind(null));

export function wrong() { return query(`LOCK TABLE ${WRONG.join(', ')}`); }
export function bound() { return query(`LOCK TABLE ${BOUND.join(', ')}`); }
export function shadowed(assertIdent: (value: string) => string) {
  const tables = RAW.map(assertIdent);
  return query(`LOCK TABLE ${tables.join(', ')}`);
}
