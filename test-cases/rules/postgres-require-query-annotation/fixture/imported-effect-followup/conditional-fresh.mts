import sql from 'sql-template-strings';
import { read } from '@bench/db';
import { addFilter } from './barrel.mjs';
import { addFilter as directFilter } from './helpers.mjs';
import { missingFilter } from './unresolved.mjs';
import { rewriteHead } from './helpers.mjs';

export function accepted(flag) {
  const query = flag ? sql`/* yes */ SELECT 1` : sql`/* no */ SELECT 2`;
  addFilter(query);
  read(query);
}
export function unsafe(flag) {
  const query = flag ? sql`/* yes */ SELECT 1` : sql`/* no */ SELECT 2`;
  rewriteHead(query);
  read(query);
}
export function direct(flag) {
  const query = flag ? sql`/* yes */ SELECT 1` : sql`/* no */ SELECT 2`;
  directFilter(query);
  read(query);
}
export function partiallyUnresolved(flag) {
  const query = flag ? sql`/* yes */ SELECT 1` : sql`/* no */ SELECT 2`;
  addFilter(query);
  missingFilter(query);
  read(query);
}
