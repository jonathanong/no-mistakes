import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
class Api {
  rebind(client) { box.client = client; }
}
function write() { box.client.get = replacement; }
const api = new Api();
api.rebind(replica);
const NotAClass = primary;
const missing = new NotAClass();
const anonymous = new (function () {})();
void missing;
void anonymous;
write();
replica.get('mutated');
primary.get('old');
unrelated.get('unrelated');
