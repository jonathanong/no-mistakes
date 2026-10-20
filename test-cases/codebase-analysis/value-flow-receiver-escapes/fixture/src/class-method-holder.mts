import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: primary };
class Api {
  rebind(client) { box.client = client; }
}
function write() { box.client.get = replacement; }
declare const flag: boolean;
const holder = {
  api: new Api(),
  other: 1,
  [flag]: new Api(),
  method() {},
  ...{},
};
holder.api.rebind(replica);
write();
replica.get('changed');
unrelated.get('control');
