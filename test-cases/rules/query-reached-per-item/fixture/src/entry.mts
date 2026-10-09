import { lookup, batch, cycle } from './helpers.mjs';
import { read } from 'configured-db';
const items = [1, 2];
export async function handler(tx: {query: Function}) {
  for (const item of items) lookup(item);
  for (let i = 0; i < items.length; i++) read(items[i]);
  for (const key in items) lookup(key);
  while (items.length) { lookup(items.pop()); }
  do { lookup(items.pop()); } while (items.length);
  await Promise.all(items.map(item => lookup(item)));
  items.forEach(lookup);
  items.flatMap(item => lookup(item));
  items.filter(item => lookup(item));
  items.some(item => lookup(item));
  items.every(item => lookup(item));
  items.reduce((value, item) => lookup(item), null);
  items.find(item => lookup(item));
  const promises = [];
  for (const item of items) promises.push(lookup(item));
  await Promise.allSettled(promises);
  for (const item of items) tx.query(item);
  for (const item of items) cycle(item);
  for (const item of items) batch([item]);
  batch(items.map(item => lookup(item)));
  // Declaring a dormant nested helper is not executing it per item.
  for (const item of items) { function dormant() { read(item); } }
  // Ordinary paging without a configured sink stays quiet.
  while (items.length) { await nextPage(); }
  lookup(items[0]);
}
function localLookup(item: unknown) { return read(item); }
export function callbackShapes() {
  items.map(function (item) { return localLookup(item); });
  items.forEach(localLookup);
  const dynamic = unknownCallback;
  items.forEach(dynamic);
  items.map();
  items.map(...items);
  factory().map(item => lookup(item));
  // Sequential paging with a configured sink requires a justified suppression.
  // no-mistakes-disable-next-line query-reached-per-item -- cursor pages depend on previous page
  while (items.length) read(items.pop());
  for (const item of items) {
    read(item); // no-mistakes-disable-line query-reached-per-item -- ordered writes
  }
}
import { transactional, mixed, onlyBatch, batchAndRead } from './helpers.mjs';
const batchAlias = batch;
export function transactionAndBatchCases(tx: {query: Function}) {
  for (const item of items) transactional(tx);
  for (const item of items) mixed(tx);
  for (const item of items) onlyBatch([item]);
  for (const item of items) batchAlias([item]);
  for (const item of items) batchAndRead([item]);
  items.map(() => batchAlias(items));
}
for (const item of items) read(item);
const db = { read };
for (const item of items) db.read(item);
export function aliasedBatchWrapper() { batchAlias(items.map(item => lookup(item))); }
export function shadowedBatchAlias() {
  const batchAlias = (values: unknown[]) => values;
  batchAlias(items.map(item => lookup(item)));
}
export function unsupportedCallbacksAndLoopHeaders() {
  items.map(null);
  getCallable()();
  for (let i = read(0); i < 2; i++) lookup(i);
}
export function repeatedCalls() { for (const item of items) { lookup(item); lookup(item); } }
export function repeatedCallbacks() { items.map(lookup); items.map(lookup); }
export function configuredNamedMemberCallback(client: {read: Function}) { items.map(client.read); }
const lookupAlias = lookup;
items.map(lookupAlias);
export function namedAliasCallbacks() { items.forEach(lookupAlias); }
