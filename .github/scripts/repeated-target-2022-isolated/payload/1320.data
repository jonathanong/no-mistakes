import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
const saved = holder.box;
holder.box = { client };
saved.client.get = replacement;
function globalSavedAfterFresh(value) { value.get('key'); }
globalSavedAfterFresh(client);
