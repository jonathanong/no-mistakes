import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// A loop can invoke one writer owner more than once despite one syntactic call.
function write() { box.client.get = replacement; box = { client: replica }; }
for (const iteration of [1, 2]) { write(); }
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
  value.info('primary sibling');
  changed.info('replica sibling');
}
consume(primary, replica);
