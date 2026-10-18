import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// The root is acquired by write, while the nested closure owns the mutation.
function write() { const b = box; return () => { b.client.get = replacement; }; }
function rebind() { box = { client: replica }; }
rebind();
write()();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
