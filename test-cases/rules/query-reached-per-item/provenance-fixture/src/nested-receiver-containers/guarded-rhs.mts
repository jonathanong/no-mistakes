import { primary as client } from '@vendor/valkey';
function replacement() {}
const box = { client };
if (flag) { other = (box.client = {}); box.client.get = replacement; }
function guardedRhs(value) { value.get('key'); }
guardedRhs(client);
