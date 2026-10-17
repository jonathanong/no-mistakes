import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
const saved = holder.box;
holder.box = { client };
holder.box.client = {};
saved.client.get = replacement;
function globalSavedBeforeFresh(value) { value.get('key'); }
globalSavedBeforeFresh(client);
