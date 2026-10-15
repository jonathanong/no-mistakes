import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = { x: (holder.box.other = 1) };
holder.box.client.get = replacement;
function globalSiblingRhs(value) { value.get('key'); }
globalSiblingRhs(client);
