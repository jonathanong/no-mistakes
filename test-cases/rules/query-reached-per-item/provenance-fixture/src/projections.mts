import { read } from './psql.mjs';
import { primary } from '@vendor/valkey';
const items = ['one'];
function applyDefault(client = primary) { return client.get('key'); }
for (const item of items) { applyDefault.apply(null, undefined); }
const client = { query: read };
const copied = client.query;
const { query: destructured } = client;
client.query = unrelated;
for (const item of items) { copied(item); }
for (const item of items) { destructured(item); }
for (const item of items) { client.query(item); }
const { query: missing = read } = {};
for (const item of items) { missing(item); }
const { query: explicit = read } = { query: undefined };
for (const item of items) { explicit(item); }
const { query: unknown = read } = dynamic;
for (const item of items) { unknown(item); }
function shadowed(undefined: unknown) {
  function applyShadow(client = primary) { return client.get('key'); }
  for (const item of items) { applyShadow.apply(null, undefined); }
}
shadowed(unrelated);
