import { primary as client } from '@vendor/valkey';
function replacement() {}
const items = ['one'];
const box = { wrapper: { client } };
box.wrapper = { client };
box.wrapper.client.set = replacement;
function retainedSibling(value) { value.get('key'); }
for (const item of items) { retainedSibling(client); }
