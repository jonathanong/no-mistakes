import { query } from '@example/db';
import { assertIdent } from './wrong-validator-barrel';
const RAW = ['orders', 'refunds'];
const TABLES = RAW.map(assertIdent);
export function wrongBarrel() { return query(`LOCK TABLE ${TABLES.join(', ')}`); }
