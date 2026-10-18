import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const copied = replica.get;
const copiedPrimary = primary.get;
const captured = box;
function write() { box.client.get = replacement; }
function rebind() { box = { client: replica }; }
rebind();
write();
function consume(value, changed, other, method, original) {
  value.info('sibling');
  changed.info('sibling');
  other.get('unrelated');
  method('copied');
  original('copied-primary');
  captured.client.info('captured-sibling');
}
consume(primary, replica, unrelated, copied, copiedPrimary);
