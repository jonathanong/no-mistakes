import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
(class {
  @opaque method() {}
  static first = 0;
  // This key runs before decorator application, despite its source position.
  [(holder.box.client.get = replacement, 'key')]() {}
});
function opaque() { holder.box = { client }; }
function scan(value) { value.get('key'); }
scan(client);
