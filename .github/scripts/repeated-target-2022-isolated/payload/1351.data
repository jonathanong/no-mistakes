import { primary as live, primary as snapshot, primary as supplied } from '@vendor/valkey';
function replacement() {}
const items = ['one'];
const selected = { client: live };
const other = { client: live };
function liveDefault({ client } = selected) { other.client.get = replacement; client.get('key'); }
for (const item of items) { liveDefault(); }

const snapshotSelected = { client: snapshot };
const snapshotOther = { client: snapshot };
function copiedDefault({ client, get = client.get } = snapshotSelected) { snapshotOther.client.get = replacement; get('key'); }
for (const item of items) { copiedDefault(); }

const suppliedBox = { client: supplied };
function mutateSupplied() { suppliedBox.client.get = replacement; }
function originalArgument(client) { client.get('key'); }
for (const item of items) { originalArgument(supplied); }

import { primary as globalLate } from '@vendor/valkey';
const globalSelected = { client: globalLate };
const globalOther = { client: globalLate };
globalOther.client.get = replacement;
function globalDefault({ client } = globalSelected) { client.get('key'); }
for (const item of items) { globalDefault(); }

import { primary as globalEarly } from '@vendor/valkey';
const earlySelected = { client: globalEarly };
const earlyOther = { client: globalEarly };
function earlyGlobalDefault({ client } = earlySelected) { client.get('key'); }
for (const item of items) { earlyGlobalDefault(); }
earlyOther.client.get = replacement;
