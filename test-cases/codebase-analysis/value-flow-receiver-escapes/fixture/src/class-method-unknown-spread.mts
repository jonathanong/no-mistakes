import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function write() {
  box.client.get = replacement;
}
class Api {
  rebind(a, c) {
    box.client = c;
  }
}
declare const rest: any[];
new Api().rebind(...rest, replica);
write();
replica.get('changed');
unrelated.get('control');
