import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
const copied = client.get;
holder.box.client = { x: (holder.box = { client }) };
holder.box.client.get = replacement;
function globalCopiedMethod(fn) { fn('key'); }
globalCopiedMethod(copied);
