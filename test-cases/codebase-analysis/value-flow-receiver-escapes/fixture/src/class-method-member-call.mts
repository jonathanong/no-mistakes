import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
class Api {
  rebind(client) { box.client = client; }
}
function write() { box.client.get = replacement; }
const api = new Api();
api.rebind.call(api, replica);
write();
replica.get('changed');
unrelated.get('control');
