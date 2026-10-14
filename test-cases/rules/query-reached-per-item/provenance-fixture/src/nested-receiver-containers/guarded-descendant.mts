import { primary as client } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: { client } };
if (flag) { box.wrapper = { client }; box.wrapper.client.get = replacement; }
function guardedDescendant(value) { value.get('key'); }
guardedDescendant(client);
