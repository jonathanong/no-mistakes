import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
const saved = holder.box;
holder.box = { client };
saved.client = {};
saved.client.get = replacement;
function globalSavedOldWrapper(value) { value.get('key'); }
globalSavedOldWrapper(client);
