import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
holder.box.other = { client };
holder.box.client.get = replacement;
function globalSiblingAfter(value) { value.get('key'); }
globalSiblingAfter(client);
