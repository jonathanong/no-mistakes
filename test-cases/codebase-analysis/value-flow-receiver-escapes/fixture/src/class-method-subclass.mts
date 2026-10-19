import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
class Api {
  rebind(client) { box.client = client; }
}
class Sub extends Api {}
function write() { box.client.get = replacement; }
new Sub().rebind(replica);
// A missing method on a class with no base must not invent a callee.
new Api().nope(replica);
write();
replica.get('changed');
unrelated.get('control');
