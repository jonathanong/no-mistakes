import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = { [rebind()]: 1 };
holder.box.client.get = replacement;
function rebind() { holder.box = { client }; return 'x'; }
function globalComputedRhsCall(value) { value.get('key'); }
globalComputedRhsCall(client);
