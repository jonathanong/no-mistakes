import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// Unknown computed stores overlap client; body visitation cannot make replica.get exact.
function rebind(key) { box[key] = replica; }
function write() { box.client.get = replacement; }
rebind('client');
write();
function consume(value, changed) {
  // Foreign writer summaries conservatively lose the untouched primary.get precision.
  value.get('primary');
  changed.get('replica');
  value.info('primary sibling');
  changed.info('replica sibling');
}
consume(primary, replica);
