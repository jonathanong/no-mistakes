import { query } from '@example/db';
const OK = ['orders', 'refunds'];
let MUTABLE = ['orders', 'refunds'];
const MUTATED = ['orders', 'refunds'];
MUTATED.push(loadTable());

export function conditional(table: string, enabled: boolean) {
  if (enabled) {
    if (!OK.includes(table)) throw new Error('bad table');
  }
  return query(`SELECT 1 FROM ${table}`);
}

export function late(table: string) {
  query(`SELECT 1 FROM ${table}`);
  if (!OK.includes(table)) throw new Error('bad table');
}

export function mutable(table: string) {
  if (!MUTABLE.includes(table)) throw new Error('bad table');
  return query(`SELECT 1 FROM ${table}`);
}

export function mutated(table: string) {
  if (!MUTATED.includes(table)) throw new Error('bad table');
  return query(`SELECT 1 FROM ${table}`);
}
