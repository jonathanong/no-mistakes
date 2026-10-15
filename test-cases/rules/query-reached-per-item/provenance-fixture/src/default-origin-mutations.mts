import { primary, session } from '@vendor/valkey-default-origin-mutations';
const items = ['one'];
const opts = { client: primary };
function rootDefault({ client } = opts) { opts.client.get = unrelated; client.get('key'); }
for (const item of items) { rootDefault(); }
const intermediate = { client: primary };
function nestedDefault({ box: { client } = intermediate } = {}) { intermediate.client.get = unrelated; client.get('key'); }
for (const item of items) { nestedDefault(); }
function supplied(client = primary) { primary.info = unrelated; client.info('key'); }
for (const item of items) { supplied(session); }
const unused = { client: primary };
function suppliedPattern({ client } = unused) { unused.client.get = unrelated; client.info('key'); }
for (const item of items) { suppliedPattern({ client: session }); }
const copied = { client: primary };
function copyMethod({ client, get = client.get } = copied) { copied.client.get = unrelated; get('key'); }
for (const item of items) { copyMethod(); }
const replaced = { client: primary };
function replaceField({ client } = replaced) { replaced.client = {}; client.get('key'); }
for (const item of items) { replaceField(); }
const chains = { client: primary };
function chain({ client } = chains, alias = client, next = alias) { chains.client.get = unrelated; next.get('key'); }
for (const item of items) { chain(); }
const missing = {};
function replacedOrigin({ client = session } = missing) { missing.client = primary; client.info('key'); }
for (const item of items) { replacedOrigin(); }
const provided = { client: primary };
function providedIntermediate({ box: { client } = provided }) { provided.client.get = unrelated; client.info('key'); }
for (const item of items) { providedIntermediate({ box: { client: session } }); }
const firstSource = {};
const secondSource = { client: session };
function switchOrigin({ box: { client } = secondSource } = firstSource) { firstSource.box = {}; client.info('key'); }
for (const item of items) { switchOrigin(); }
