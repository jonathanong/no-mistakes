import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// The later rebind in the acquisition owner cannot change the saved object.
function write() {
  const b = box;
  const mutate = () => { b.client.get = replacement; };
  box = { client: replica };
  return mutate;
}
write()();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
