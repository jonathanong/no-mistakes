import { primary as client } from '@vendor/valkey';
function replacement() {}
const box = { client };
function restore() { box.client = client; }
if (flag) { box.client = {}; box.client.get = replacement; }
function guardedForeign(value) { value.get('key'); }
guardedForeign(client);
