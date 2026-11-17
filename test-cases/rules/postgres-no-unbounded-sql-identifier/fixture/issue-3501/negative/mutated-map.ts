import { query } from '@example/db';
import { assertIdent } from '../validators';

const RAW_TABLES = ['orders', 'refunds'];
RAW_TABLES.push(loadTable());
const TABLES = RAW_TABLES.map(assertIdent);

export function locks() {
  return query(`LOCK TABLE ${TABLES.join(', ')}`);
}
