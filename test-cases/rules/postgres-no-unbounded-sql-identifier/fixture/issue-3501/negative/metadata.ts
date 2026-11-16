import { query } from '@example/db';
const ITEMS = [{ k: 'order', table: 'orders' }, { k: 'refund', table: 'refunds' }];
let MUTABLE = [{ k: 'order', table: 'orders' }];
const OPEN = [{ k: 'order', table: process.env.TABLE }];

export function unchecked(key: string) {
  const item = ITEMS.find((entry) => entry.k === key);
  return query(`SELECT 1 FROM ${item.table}`);
}

export function conditional(key: string, enabled: boolean) {
  const item = ITEMS.find((entry) => entry.k === key);
  if (enabled && !item) throw new Error('unknown');
  return query(`SELECT 1 FROM ${item.table}`);
}

export function mutable(key: string) {
  const item = MUTABLE.find((entry) => entry.k === key);
  if (!item) throw new Error('unknown');
  return query(`SELECT 1 FROM ${item.table}`);
}

export function open(key: string) {
  const item = OPEN.find((entry) => entry.k === key);
  if (!item) throw new Error('unknown');
  return query(`SELECT 1 FROM ${item.table}`);
}
