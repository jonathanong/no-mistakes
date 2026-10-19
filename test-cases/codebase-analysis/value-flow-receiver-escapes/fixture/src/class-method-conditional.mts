import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
const box = { client: primary };
const other = { client: untouched };
class Api {
  rebind(c) {
    box.client = c;
  }
}
function write() {
  box.client.get = replacement;
}
// `unrelated` is not an argument, so its call stays known.
new Api().rebind(flag ? other : replica);
write();
replica.get('changed');
unrelated.get('control');
