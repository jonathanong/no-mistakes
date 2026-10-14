import { primary as client } from '@vendor/valkey';
function replacement() {}
let box = { client };
if (flag) { box.client = {}; box.client = client; box.client.get = replacement; }
function guardedRestore(value) { value.get('key'); }
guardedRestore(client);
