import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// A module capture keeps the original object through both callable layers.
const b = box;
function write() { return () => { b.client.get = replacement; }; }
function rebind() { box = { client: replica }; }
rebind();
write()();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
