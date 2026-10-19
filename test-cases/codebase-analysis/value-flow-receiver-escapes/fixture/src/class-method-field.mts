import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
const box = { client: primary };
class Api {
  // A non-callable field is not a method provider.
  label = primary;
  declared: string;
  [flag]: primary;
  rebind = (client) => { box.client = client; };
}
function write() { box.client.get = replacement; }
new Api().rebind(replica);
write();
replica.get('changed');
unrelated.get('control');
