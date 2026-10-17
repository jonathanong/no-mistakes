import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const captured = box;
// Rebinding box never changes the object already held by captured.
function write() { captured.client.get = replacement; }
function rebind() { box = { client: replica }; }
rebind();
write();
function consume(changed, stable) {
  changed.get('primary');
  stable.get('replica');
}
consume(primary, replica);
