import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
const saved = holder.box.client;
holder.box = { client };
saved.get = replacement;
function globalSavedClient(value) { value.get('key'); }
globalSavedClient(client);
