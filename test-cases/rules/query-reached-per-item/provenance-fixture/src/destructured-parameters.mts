import { primary, session } from '@vendor/valkey';
const items = ['one'];
function object({ client }) { client.get('key'); }
for (const item of items) { object({ client: primary, unused: item }); }
function array([, client]) { client.get('key'); }
for (const item of items) { array([undefined, primary]); }
function nested({ box: { client = primary } = {} } = {}) { client.get('key'); }
for (const item of items) { nested(); }
function conflict({ client }) { client.get('key'); }
conflict({ client: primary });
for (const item of items) { conflict({ client: session }); }
function mutated({ client }) { client.get = unrelated; client.get('key'); }
for (const item of items) { mutated({ client: primary }); }
function reassigned({ client }) { client = session; client.get('key'); }
for (const item of items) { reassigned({ client: primary }); }
function earlier({ first = primary, client = first } = {}) { client.get('key'); }
for (const item of items) { earlier(); }
function later({ client = next, next = primary } = {}) { client.get('key'); }
for (const item of items) { later(); }
function earlierParameter(first = primary, { client = first } = {}) { client.get('key'); }
for (const item of items) { earlierParameter(); }
function laterParameter({ client = next } = {}, next = primary) { client.get('key'); }
for (const item of items) { laterParameter(); }
function computed({ [key]: client }) { client.get('key'); }
for (const item of items) { computed({ key: primary }); }
function rest({ ...client }) { client.get('key'); }
for (const item of items) { rest(primary); }
function arrayRest([ ...clients ]) { clients[0].get('key'); }
for (const item of items) { arrayRest([primary]); }
function arrayDefault([client = primary] = []) { client.get('key'); }
for (const item of items) { arrayDefault([,]); }
function spread([client]) { client.get('key'); }
for (const item of items) { spread([...unknown, primary]); }
function unrelatedObject({ client }) { client.get('key'); }
for (const item of items) { unrelatedObject({ client: {} }); }
// Unused fields differ, but the projected client agrees.
object({ client: primary, unused: 'different' });
function nestedLater({ box: { client } = { client: later }, later = primary } = {}) { client.get('key'); }
for (const item of items) { nestedLater(); }
function rootLater({ client, later } = { client: later, later: primary }) { client.get('key'); }
for (const item of items) { rootLater(); }
import { projected } from './destructured-export.mjs';
for (const item of items) { projected({ client: primary }); }
const arrow = ([client]) => client.get('key');
for (const item of items) { arrow([primary]); }
// Numeric object keys do not make an object iterable.
function nonIterable([client]) { client.get('key'); }
for (const item of items) { nonIterable({ 0: primary }); }
function aliasMutation({ client, alias = client }) { client.get = unrelated; alias.get('key'); }
for (const item of items) { aliasMutation({ client: primary }); }
function copiedDefault({ client, get = client.get }) { client.get = unrelated; get('key'); }
for (const item of items) { copiedDefault({ client: primary }); }
// Array length exists even when the array has no elements.
function arrayLength({ length: client = primary }) { client.get('key'); }
for (const item of items) { arrayLength([]); }
