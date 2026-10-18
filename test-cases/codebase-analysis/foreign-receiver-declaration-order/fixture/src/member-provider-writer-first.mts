import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write() { box.client.get = replacement; }
function rebind() { box.client = replica; }
rebind();
write();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
  value.info('primary sibling');
  changed.info('replica sibling');
}
consume(primary, replica);
