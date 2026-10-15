import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = { x: opaque() };
holder.box.client.get = replacement;
function globalUnknownCall(value) { value.get('key'); }
globalUnknownCall(client);
