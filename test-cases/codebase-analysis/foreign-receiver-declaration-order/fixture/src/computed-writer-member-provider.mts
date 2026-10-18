import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// A dynamic receiver projection overlaps every static member provider prefix.
function write(key) { box[key].get = replacement; }
function rebind() { box.client = replica; }
rebind();
write('client');
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
  value.info('primary sibling');
  changed.info('replica sibling');
}
consume(primary, replica);
