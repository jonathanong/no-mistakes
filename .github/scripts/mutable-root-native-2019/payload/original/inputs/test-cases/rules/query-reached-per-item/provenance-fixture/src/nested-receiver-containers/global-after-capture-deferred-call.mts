import { primary as client } from '@vendor/valkey';
function replacement() {}
function opaque(value) { value.box = { client }; }
const holder = { box: { client } };
holder.box.client = {};
function deferred() { opaque(holder); }
holder.box.client.get = replacement;
function globalAfterCaptureDeferredCall(value) { value.get('key'); }
globalAfterCaptureDeferredCall(client);
