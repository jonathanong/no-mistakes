import { read as load, write as save, query as execute } from '@data-stores/psql';
import * as postgres from '@data-stores/psql';
import { valkey as cache } from '@data-stores/valkey';
import defaultDb from '@vendor/default-db';
import { lookup, cachedLookup, unrelated } from './helpers.mjs';
import { forwarded } from './external-barrel.mjs';
import { read as repositoryRead } from './repository.mjs';
import { read as otherRepositoryRead } from './repository-barrel.mjs';
import { read as streamRead, write as streamWrite } from 'node:stream';
const items = [1, 2];
for (const item of items) load(item);
for (const item of items) save(item);
for (const item of items) execute(item);
for (const item of items) postgres.read(item);
const loadAlias = load;
for (const item of items) loadAlias(item);
const memberAlias = postgres.query;
for (const item of items) memberAlias(item);
items.map(load);
items.map(postgres.read);
for (const item of items) cache.hget(item);
const cacheAlias = cache;
for (const item of items) cacheAlias.get(item);
items.forEach(cache.get);
for (const item of items) defaultDb.query(item);
for (const item of items) forwarded(item);
for (const item of items) repositoryRead(item);
for (const item of items) lookup(item);
items.map(cachedLookup);
// Targets match module bindings, never these same-name methods or parameters.
for (const item of items) console.write(item);
for (const item of items) streamRead(item);
for (const item of items) streamWrite(item);
for (const item of items) unrelated(item);
// Repository selectors retain the exact direct import-specifier boundary.
for (const item of items) otherRepositoryRead(item);
export function localShadows(load: Function, postgres: { read: Function }, cache: { get: Function }, defaultDb: { query: Function }) {
  for (const item of items) load(item);
  for (const item of items) postgres.read(item);
  for (const item of items) cache.get(item);
  for (const item of items) defaultDb.query(item);
}
function read(item: unknown) { return item; }
function write(item: unknown) { return item; }
function query(item: unknown) { return item; }
for (const item of items) read(item);
for (const item of items) write(item);
for (const item of items) query(item);
