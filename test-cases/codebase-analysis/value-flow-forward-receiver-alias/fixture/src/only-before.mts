import { primary, replica } from '@vendor/forward-only-before';
function replacement() {}
let box = { client: primary };
function write() { before.client.get = replacement; }
const before = box;
box = { client: replica };
const after = box;
write();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
