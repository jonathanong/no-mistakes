import { primary as client } from '@vendor/valkey';
function replacement() {}
let box = { client };
if (flag) { box.client = {}; box[key] = client; box.client.get = replacement; }
function guardedDynamic(value) { value.get('key'); }
guardedDynamic(client);
