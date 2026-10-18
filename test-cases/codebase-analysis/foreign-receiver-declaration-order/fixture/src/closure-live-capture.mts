import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// A mutable captured binding remains live after the nested closure is created.
function write() {
  let b = box;
  const mutate = () => { b.client.get = replacement; };
  b = { client: replica };
  return mutate;
}
write()();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
