import { primary } from '@vendor/valkey';
import { beginTransaction, read } from './psql.mjs';
const items = ['one'];
function reassigned(client = primary) { client = unrelated; return client.get('key'); }
function shadowed(primary: any) { const client = primary; return client.get('key'); }
function conflicting(client = primary) { return client.get('key'); }
// A shared graph edge requires agreeing callers; mixed receivers stay unknown.
for (const item of items) { conflicting(primary); conflicting(unrelated); reassigned(); shadowed(unrelated); }
async function bindings() {
  let executor = await beginTransaction();
  for (const item of items) { executor(item); }
  executor = unrelated;
  for (const item of items) { executor(item); }
}
function objectFactory() { return { execute: read }; }
const { execute: fromObject } = objectFactory();
for (const item of items) { fromObject(item); }
function unsupportedFactory() { return { ...unknownObject, execute: read }; }
const { execute: unknownExecutor } = unsupportedFactory();
for (const item of items) { unknownExecutor(item); }
function parameterCycle(client: any) { parameterCycle(client); return client.get('key'); }
for (const item of items) { parameterCycle(unrelated); }
function defaultCaller(client = primary) { return client.get('key'); }
for (const item of items) { defaultCaller(undefined); }
function makeReader(client = primary) { return () => client.get('key'); }
const capturedReader = makeReader();
for (const item of items) { capturedReader(); }
function scalarFactory() { return 1; }
const scalar = scalarFactory();
for (const item of items) { scalar(); }
function branchFactory(flag: boolean) { if (flag) return read; return unrelated; }
const branch = branchFactory(true);
for (const item of items) { branch(item); }
function returnCycle() { return returnCycle(); }
const recursive = returnCycle();
for (const item of items) { recursive(item); }
const namespaceCache = { primary };
const cacheAlias = namespaceCache.primary;
for (const item of items) { cacheAlias.get(item); }
