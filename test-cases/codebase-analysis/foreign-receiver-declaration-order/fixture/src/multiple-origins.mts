import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write() { box.client.get = replacement; }
function primaryRoot() { box = { client: primary }; }
function replicaRoot() { box = { client: replica }; }
primaryRoot();
replicaRoot();
write();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
