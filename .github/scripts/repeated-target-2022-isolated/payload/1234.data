import { read } from './psql.mjs';
import { primary } from '@vendor/valkey';
import { begin } from '@vendor/opaque-database';
const items = ['one'];
const client = { query: read };
const alias = client;
alias.query = unrelated;
for (const item of items) { client.query(item); }
function exhaustive(flag: boolean) { if (flag) return read; else return read; }
const fromBranches = exhaustive(true);
for (const item of items) { fromBranches(item); }
function callWrong(client = primary) { return client.get('key'); }
function callKnown(client = primary) { return client.get('key'); }
for (const item of items) { callWrong.call(undefined, unrelated); }
for (const item of items) { callKnown.call(undefined, primary); }
function applyWrong(client = primary) { return client.get('key'); }
function applyKnown(client = primary) { return client.get('key'); }
for (const item of items) { applyWrong.apply(undefined, [unrelated]); }
for (const item of items) { applyKnown.apply(undefined, [primary]); }
function applyDynamic(client = primary) { return client.get('key'); }
for (const item of items) { applyDynamic.apply(undefined, dynamicArguments); }
const callTransaction = begin.call(undefined);
for (const item of items) { callTransaction(item); }
class WrongService { constructor(client = primary) { client.get('key'); } }
class DefaultService { constructor(client = primary) { client.get('key'); } }
function constructWrong() { return new WrongService(unrelated); }
function constructDefault() { return new DefaultService(); }
for (const item of items) { constructWrong(); }
for (const item of items) { constructDefault(); }
var executor = begin();
var executor = unrelated;
for (const item of items) { executor(item); }
