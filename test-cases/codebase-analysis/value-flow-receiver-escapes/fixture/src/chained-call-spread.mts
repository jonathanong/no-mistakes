import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
class Api {
  rebind(c) {
    box.client = c;
  }
  other(_c) {}
}
function write() {
  box.client.get = replacement;
}
// Both calls start at `new`. The later spread must not erase `replica`.
new Api().rebind(...[replica]).other(...[untouched]);
write();
replica.get('changed');
primary.get('old');
untouched.get('control');
unrelated.get('unrelated');
