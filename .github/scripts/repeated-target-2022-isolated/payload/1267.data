import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
holder.box.client &&= { x: (holder.box = { client }) };
holder.box.client.get = replacement;
function globalAncestorLogical(value) { value.get('key'); }
globalAncestorLogical(client);
