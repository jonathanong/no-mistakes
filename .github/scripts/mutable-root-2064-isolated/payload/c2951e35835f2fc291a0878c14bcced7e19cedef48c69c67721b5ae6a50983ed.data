import { primary as client } from '@vendor/valkey';
function replacement() {}
function opaque(value) { value.box = { client }; }
const holder = { box: { client } };
holder.box.client = {};
opaque(holder);
holder.box.client.get = replacement;
function globalAfterCaptureCall(value) { value.get('key'); }
globalAfterCaptureCall(client);
