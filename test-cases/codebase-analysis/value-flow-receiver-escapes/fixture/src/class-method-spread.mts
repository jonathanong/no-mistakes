import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
class Api {
  rebind(c) {
    box.client = c;
  }
}
function write() {
  box.client.get = replacement;
}
new Api().rebind(...[replica]);
write();
replica.get('changed');
unrelated.get('control');
