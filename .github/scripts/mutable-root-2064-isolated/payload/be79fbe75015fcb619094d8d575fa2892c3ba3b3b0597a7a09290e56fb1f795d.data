import { primary as client } from '@vendor/valkey';
function replacement() {}
const box = { client };
if (flag) { box.client ||= {}; box.client.get = replacement; }
function guardedLogical(value) { value.get('key'); }
guardedLogical(client);
