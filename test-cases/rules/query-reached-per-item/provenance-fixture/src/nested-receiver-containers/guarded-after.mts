import { primary as client } from '@vendor/valkey';
function replacement() {}
let box = { client };
if (flag) { box.client = {}; }
box.client.get = replacement;
function guardedAfter(value) { value.get('key'); }
guardedAfter(client);
