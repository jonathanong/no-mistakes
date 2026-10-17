import { read } from './psql.mjs';
import { primary } from '@vendor/valkey';
import { begin } from '@vendor/opaque-database';
const items = ['one'];
function choose(kind) { switch (kind) { case 'a': case 'b': return read; default: return read; } }
const chosen = choose('a');
for (const item of items) { chosen(item); }
function fallthrough(kind) { switch (kind) { case 'a': break; default: return read; } }
const maybe = fallthrough('a');
for (const item of items) { maybe(item); }
function voidDefault(client = primary) { client.get('key'); }
for (const item of items) { voidDefault(void 0); }
const client = { query: read };
const box = { client };
box.client.query = unrelated;
for (const item of items) { client.query(item); }
const sql = begin();
for (const item of items) { sql`SELECT ${item}`; }
const unchanged = { query: read };
const replaced = { client: unchanged };
replaced.client = unrelated;
for (const item of items) { unchanged.query(item); }
const nestedClient = { query: read };
const outer = { inner: { client: nestedClient } };
const outerAlias = outer;
outerAlias.inner.client.query = unrelated;
for (const item of items) { nestedClient.query(item); }
