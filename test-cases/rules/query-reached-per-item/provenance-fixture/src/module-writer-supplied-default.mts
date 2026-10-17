import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write() { box.client.get = replacement; }
box = { client: replica };
write();
// Supplied and default receiver origins share the same guarded write history.
function consume(value = replica) {
  value.get('changed');
  value.info('untouched sibling');
}
consume();
consume(replica);
