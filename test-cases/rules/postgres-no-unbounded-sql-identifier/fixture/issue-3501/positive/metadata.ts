import { query } from '@example/db';
const ITEMS = [{ k: 'order', table: 'orders' }, { k: 'refund', table: 'refunds' }];

function tableFor(key: string) {
  const item = ITEMS.find((entry) => entry.k === key);
  if (!item) throw new Error('unknown');
  return item.table;
}

export function selected() {
  const table = tableFor('order');
  return query(`SELECT 1 FROM ${table}`);
}

export function localLookup(key: string) {
  const item = ITEMS.find((entry) => entry.k === key);
  if (!item) throw new Error('unknown');
  const table = item.table;
  return query(`SELECT 1 FROM ${table}`);
}
