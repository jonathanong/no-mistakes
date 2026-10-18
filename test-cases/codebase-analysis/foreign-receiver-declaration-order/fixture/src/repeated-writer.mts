import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// The second invocation acquires the root rebound by the first invocation.
function write() { box.client.get = replacement; box = { client: replica }; }
write();
write();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
  value.info('primary sibling');
  changed.info('replica sibling');
}
consume(primary, replica);
