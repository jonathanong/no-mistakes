import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write() { box.client.get = replacement; }
function rebind() {
  box.sibling = replica;
  box.client.extra = replica;
}
rebind();
write();
// Neither a sibling nor a deeper store replaces the writer's client receiver.
function consume(value, changed) {
  value.get('primary');
  changed.get('replica remains known');
}
consume(primary, replica);
