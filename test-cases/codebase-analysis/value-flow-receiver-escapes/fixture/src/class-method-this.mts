import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
class Api {
  rebind(client) { box.client = client; }
  run() {
    this.rebind(replica);
    // An arrow still sees the class. A nested function does not.
    const go = () => { this.rebind(replica); };
    go();
    function nested() { this.rebind(replica); }
    nested();
  }
}
function write() { box.client.get = replacement; }
new Api().run();
write();
replica.get('changed');
unrelated.get('control');
