import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = { x: (holder[key] = { client }) };
holder.box.client.get = replacement;
function globalDynamicAncestor(value) { value.get('key'); }
globalDynamicAncestor(client);
