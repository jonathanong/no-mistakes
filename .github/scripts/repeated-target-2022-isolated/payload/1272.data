import { primary as client } from '@vendor/valkey';
function replacement() {}
function opaque(value) { value.box = { client }; }
opaque({});
const holder = { box: { client } };
holder.box.client = {};
holder.box.client.get = replacement;
function globalBeforeCaptureCall(value) { value.get('key'); }
globalBeforeCaptureCall(client);
