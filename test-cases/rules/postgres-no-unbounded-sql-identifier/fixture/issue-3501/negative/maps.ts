import { query } from '@example/db';
import { assertIdent } from '../validators';
import { OPEN_TABLES, MUTABLE_TABLES } from '../open-data';
const RAW = ['orders', 'refunds'];
let LOCAL_MUTABLE = ['orders', 'refunds'];
const OPEN = OPEN_TABLES.map(assertIdent);
const IMPORTED_MUTABLE = MUTABLE_TABLES.map(assertIdent);
const LOCAL = LOCAL_MUTABLE.map(assertIdent);
const CAPTURED = RAW.map(() => `COUNT(${process.env.COLUMN})`).join(',');

export function open() { return query(`LOCK TABLE ${OPEN.join(', ')}`); }
export function importedMutable() { return query(`LOCK TABLE ${IMPORTED_MUTABLE.join(', ')}`); }
export function mutable() { return query(`LOCK TABLE ${LOCAL.join(', ')}`); }
export function captured() { return query(`SELECT ${CAPTURED} FROM orders`); }
