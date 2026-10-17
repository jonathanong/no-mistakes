import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function rebind() { box = { client: replica }; }
function write() { box.client.get = replacement; }
rebind();
write();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
