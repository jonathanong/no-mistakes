import type { TransactionQuery } from 'database-client';
import { writePool } from 'database-client';
import { createClient } from './providers.mjs';

function typedItems(ids: string[], execute) {
  for (const id of ids) execute(id);
}
export function typedSource(ids: string[], execute: TransactionQuery) {
  typedItems(ids, execute);
  typedItems(ids, execute);
}
function memberItems(ids: string[], execute) {
  for (const id of ids) execute(id);
}
export async function memberSource(ids: string[]) {
  const client = await writePool.connect();
  memberItems(ids, client.query);
  memberItems(ids, client.query);
}
function objectItems(ids: string[], client) {
  for (const id of ids) client.query(id);
}
export function objectSource(ids: string[]) {
  objectItems(ids, createClient());
  objectItems(ids, createClient());
}
function boundItems(ids: string[], execute) {
  for (const id of ids) execute(id);
}
export function boundSource(ids: string[], execute: TransactionQuery) {
  boundItems(ids, execute.bind(null));
}
