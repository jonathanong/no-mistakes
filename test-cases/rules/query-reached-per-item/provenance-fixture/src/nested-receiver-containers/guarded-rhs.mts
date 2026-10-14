import { primary as client } from '@vendor/valkey';
function replacement() {}
let box = { client };
if (flag) { other = (box.client = {}); box.client.get = replacement; }
function guardedRhs(value) { value.get('key'); }
guardedRhs(client);
