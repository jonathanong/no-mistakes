import { read } from './psql.mjs';
import { primary } from '@vendor/valkey';
const items = ['one'];
async function makeRead() { return read; }
const promise = makeRead();
for (const item of items) { promise(item); }
const executor = await makeRead();
for (const item of items) { executor(item); }
const makeArrow = async () => read;
const arrowPromise = makeArrow();
for (const item of items) { arrowPromise(item); }
const arrowExecutor = await makeArrow();
for (const item of items) { arrowExecutor(item); }
function bindWrong(client = primary) { return client.get('key'); }
const wrong = bindWrong.bind(null, unrelated);
for (const item of items) { wrong(); }
function bindKnown(client = primary) { return client.get('key'); }
const known = bindKnown.bind(null, primary);
for (const item of items) { known(); }
function bindDefault(client = primary) { return client.get('key'); }
const withDefault = bindDefault.bind(null);
for (const item of items) { withDefault(); }
function bindRuntime(client = primary) { return client.get('key'); }
const runtime = bindRuntime.bind(null);
for (const item of items) { runtime(unrelated); }
function bindPartial(prefix, client = primary) { return client.get(prefix); }
const partial = bindPartial.bind(null, 'key');
const chained = partial.bind(null, primary);
for (const item of items) { chained(); }
function bindSpread(client = primary) { return client.get('key'); }
const spread = bindSpread.bind(null, ...dynamicArguments);
for (const item of items) { spread(); }
