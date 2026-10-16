import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
@opaque
class Decorated {
  // The undecorated nested class must restore the pending outer application.
  static Nested = class {};
}
holder.box.client.get = replacement;
function opaque() { holder.box = { client }; }
function scan(value) { value.get('key'); }
scan(client);
