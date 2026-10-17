import { read } from './psql.mjs';
import { primary } from '@vendor/valkey';
const items = ['one'];
function* generator() { return read; }
const iterator = generator();
for (const item of items) { iterator(item); }
const awaitedIterator = await generator();
for (const item of items) { awaitedIterator(item); }
async function* asyncGenerator() { return read; }
const asyncIterator = await asyncGenerator();
for (const item of items) { asyncIterator(item); }
function makeClient() { return { query: read }; }
const client = makeClient();
function captured() { client.query('query'); }
client.query = unrelated;
for (const item of items) { captured(); }
const stable = makeClient();
function stableCapture() { stable.query('query'); }
for (const item of items) { stableCapture(); }
const copiedClient = makeClient();
const copied = copiedClient.query;
function copiedCapture() { copied('query'); }
copiedClient.query = unrelated;
for (const item of items) { copiedCapture(); }
function laterDefault(client = later, later = primary) { client.get('key'); }
for (const item of items) { laterDefault(); }
function earlierDefault(first = primary, client = first) { client.get('key'); }
for (const item of items) { earlierDefault(); }
function providedDefault(client = later, later = primary) { client.get('key'); }
for (const item of items) { providedDefault(primary); }
const aliasedClient = makeClient();
const alias = aliasedClient;
mutate();
for (const item of items) { aliasedClient.query(item); }
alias.query = unrelated;
function mutate() { alias.query = unrelated; }
