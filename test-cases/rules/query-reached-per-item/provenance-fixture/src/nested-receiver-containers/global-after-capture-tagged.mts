import { primary as client } from '@vendor/valkey';
function replacement() {}
function opaque() { holder.box = { client }; }
const holder = { box: { client } };
holder.box.client = {};
opaque`restore`;
holder.box.client.get = replacement;
function globalAfterCaptureTagged(value) { value.get('key'); }
globalAfterCaptureTagged(client);
