import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
opaque(holder.box.client = {});
holder.box.client.get = replacement;
function opaque(value) { holder.box = { client }; }
function globalCallArgumentCapture(value) { value.get('key'); }
globalCallArgumentCapture(client);
