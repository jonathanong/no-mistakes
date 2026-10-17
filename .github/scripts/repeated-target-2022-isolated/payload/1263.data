import { primary as client } from '@vendor/valkey';
function replacement() {}
function Opaque(value) { value.box = { client }; }
const holder = { box: { client } };
holder.box.client = {};
new Opaque(holder);
holder.box.client.get = replacement;
function globalAfterCaptureNew(value) { value.get('key'); }
globalAfterCaptureNew(client);
