import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
let other = { client: replica };
function write() { box.client.get = replacement; }
// Both mutable roots participate in a cycle; resolve within the same traversal.
function rebind() { box = other; other = box; }
rebind();
write();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
