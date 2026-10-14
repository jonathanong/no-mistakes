import { primary as client } from '@vendor/valkey';
function replacement() {}
let box = { client };
if (flag) { box.client = {}; } else { box.client.get = replacement; }
function guardedOpposite(value) { value.get('key'); }
guardedOpposite(client);
