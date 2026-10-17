import { primary as client } from '@vendor/valkey';
function replacement() {}
const items = ['one'];
const box = { wrapper: { client } };
const copiedGet = client.get;
box.wrapper = { client };
box.wrapper.client.get = replacement;
function copiedBeforeContainer(fn) { fn('key'); }
for (const item of items) { copiedBeforeContainer(copiedGet); }
