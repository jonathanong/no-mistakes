import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// Declaration alone conservatively contributes receiver writes: aliases or escapes
// may invoke a writer beyond the closed direct-call evidence. Keep changed.get Unknown.
function write() { box.client.get = replacement; }
function rebind() { box = { client: replica }; }
rebind();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
