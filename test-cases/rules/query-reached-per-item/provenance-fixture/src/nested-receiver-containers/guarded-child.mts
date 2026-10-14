import { primary as client } from '@vendor/valkey';
function replacement() {}
let box = { client };
if (flag) { if (other) { box.client = {}; } box.client.get = replacement; }
function guardedChild(value) { value.get('key'); }
guardedChild(client);
