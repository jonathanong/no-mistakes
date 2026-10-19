import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
function rebind(c) {
  box.client = c;
  return other;
}
function other(_c) {}
function write() {
  box.client.get = replacement;
}
// `rebind(...)(...)` shares span.start. The outer spread must not replace the inner one.
rebind(...[replica])(...[untouched]);
write();
replica.get('changed');
primary.get('old');
untouched.get('control');
unrelated.get('unrelated');
