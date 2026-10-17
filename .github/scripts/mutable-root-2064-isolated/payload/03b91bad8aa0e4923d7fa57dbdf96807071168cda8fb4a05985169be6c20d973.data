import { primary } from '@vendor/valkey';
import { read } from './psql.mjs';
import { begin } from '@vendor/opaque-database';
import { wrappedBegin, exportedTx } from './review-wrappers.mjs';
const items = ['one'];
function supplied(client: any) { return client.get('key'); }
function makeApi() { return { run: supplied }; }
const api = makeApi();
for (const item of items) { api.run(primary); }
function defaultClient(client = primary) { return client.get('key'); }
function makeDefaultApi() { return { run: defaultClient }; }
const defaultApi = makeDefaultApi();
// This late-resolved caller must override the configured default with unknown.
for (const item of items) { defaultApi.run(unrelated); }
function bare(skip: boolean) { if (skip) return; return read; }
const maybe = bare(true);
for (const item of items) { maybe(item); }
function fallthrough(skip: boolean) { if (skip) return read; }
const maybeImplicit = fallthrough(true);
for (const item of items) { maybeImplicit(item); }
const changed = { query: read };
changed.query = unrelated;
for (const item of items) { changed.query(item); }
const wrapped = wrappedBegin();
for (const item of items) { wrapped(item); }
for (const item of items) { exportedTx(item); }
const query = begin();
for (const item of items) { query(item); }
