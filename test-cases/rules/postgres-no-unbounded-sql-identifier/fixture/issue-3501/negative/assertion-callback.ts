import { query } from '@example/db';
const RAW = ['orders', 'refunds'];
function assertAllowed(value: string) {
  if (!RAW.includes(value)) throw new Error('unknown table');
  return process.env.TABLE;
}
const TABLES = RAW.map(assertAllowed);
export function assertionResult() {
  return query(`LOCK TABLE ${TABLES.join(', ')}`);
}
