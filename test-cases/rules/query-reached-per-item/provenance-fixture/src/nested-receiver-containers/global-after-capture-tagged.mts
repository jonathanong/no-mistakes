import { primary as client } from '@vendor/valkey';
function replacement() {}
function opaque(strings, value) { value.box = { client }; }
const holder = { box: { client } };
holder.box.client = {};
opaque`${holder}`;
holder.box.client.get = replacement;
function globalAfterCaptureTagged(value) { value.get('key'); }
globalAfterCaptureTagged(client);
