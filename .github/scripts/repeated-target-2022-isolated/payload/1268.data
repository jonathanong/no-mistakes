import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = { x: (holder.box = { client }) };
holder.box.client.get = replacement;
function globalAncestorRhs(value) { value.get('key'); }
globalAncestorRhs(client);
