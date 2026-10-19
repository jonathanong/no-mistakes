import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
class Api {
  rebind(client) { box.client = client; }
  // No bare parameter, so the call has no origin slot.
  ping() {}
}
function write() { box.client.get = replacement; }
// An extra argument has no parameter slot. A non-class `new` is not a method provider.
new Api().rebind(replica, unrelated);
new Api().ping(...[replica]);
const NotAClass = primary;
new NotAClass().rebind(replica);
new (function () {})().rebind(replica);
write();
replica.get('mutated');
primary.get('old');
primary.info('primary sibling');
replica.info('replica sibling');
unrelated.get('unrelated');
