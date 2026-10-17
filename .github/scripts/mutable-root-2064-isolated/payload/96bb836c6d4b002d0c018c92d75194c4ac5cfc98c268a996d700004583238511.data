import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
interface Shape {}
class Base {}
(class<T> extends Base implements Shape {
  // Decorated members are applied before static fields, even when written later.
  static mutation = holder.box.client.get = replacement;
  @opaque method() {}
});
function opaque() { holder.box = { client }; }
function scan(value) { value.get('key'); }
scan(client);
