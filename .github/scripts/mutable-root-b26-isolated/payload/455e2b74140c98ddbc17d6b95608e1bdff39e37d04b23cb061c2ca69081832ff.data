import { primary as client } from '@vendor/valkey';
function replacement() {}
const items = ['one'];
const box = { wrapper: [client] };
box.wrapper = [client];
box.wrapper[0].get = replacement;
function retainedArray(value) { value.get('key'); }
for (const item of items) { retainedArray(client); }
