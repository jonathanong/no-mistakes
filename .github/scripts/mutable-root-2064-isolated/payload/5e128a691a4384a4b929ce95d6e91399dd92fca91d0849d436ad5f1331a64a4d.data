import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = { x: rebind() };
holder.box.client.get = replacement;
function rebind() { holder.box = { client }; return 0; }
function globalImmediateCall(value) { value.get('key'); }
globalImmediateCall(client);
