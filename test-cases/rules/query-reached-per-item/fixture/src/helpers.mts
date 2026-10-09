import { read } from 'configured-db';
export function lookup(item: unknown) { return inner(item); }
function inner(item: unknown) { return read(item); }
export function batch(items: unknown[]) { return items.map(lookup); }
export function cycle(item: unknown) { if (item) return lookup(item); return cycle(item); }
export function transactional(tx: {query: Function}) { return tx.query(1); }
export function mixed(tx: {query: Function}) { tx.query(1); return read(1); }
export function onlyBatch(items: unknown[]) { return batch(items); }
export function batchAndRead(items: unknown[]) { batch(items); return read(1); }
