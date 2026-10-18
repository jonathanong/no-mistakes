import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write() {
  const captured = box;
  const mutate = () => { captured.client.get = replacement; };
  // A member rebind changes the object saved by the closure; a root rebind would not.
  box.client = replica;
  return mutate;
}
write()();
function consume(changed) {
  changed.get('replica is mutated through the captured container');
  changed.info('sibling remains known');
}
consume(replica);
