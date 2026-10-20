import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
class Api {
  rebind(client) { box.client = client; }
}
function make() { return new Api(); }
function write() { box.client.get = replacement; }
make().rebind(replica);
write();
replica.get('changed');
unrelated.get('control');
