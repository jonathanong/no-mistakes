import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
class Base {
  rebind(client) { box.client = client; }
}
class Sub extends Base {
  // The override must not intercept an explicit super call.
  rebind(client) {}
  run() { super.rebind(replica); }
}
function write() { box.client.get = replacement; }
new Sub().run();
write();
replica.get('mutated by base method');
primary.info('previous receiver sibling');
replica.info('new receiver sibling');
unrelated.get('control');
