import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
class Api {
  rebind(client) { box.client = client; }
}
function write() { box.client.get = replacement; }
declare const flag: boolean;
const api = new Api();
api['rebind'](replica);
// A conditional receiver is not one class, so it cannot pick a callee.
(flag ? api : api).rebind(replica);
write();
replica.get('changed');
unrelated.get('control');
