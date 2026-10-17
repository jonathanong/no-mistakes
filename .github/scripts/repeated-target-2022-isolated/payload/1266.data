import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
holder.box = { client };
holder.box.client.get = replacement;
function globalAncestorBetween(value) { value.get('key'); }
globalAncestorBetween(client);
