import { primary, replica, unrelated } from '@vendor/forward-before';
function replacement() {}
let box = { client: primary };
// Declaration order does not execute this body before the aliases initialize.
function write() {
  before.client.get = replacement;
  after.client.get = replacement;
}
function rebind() { box = { client: replica }; }
const before = box;
rebind();
const after = box;
const copiedPrimary = primary.get;
const copiedReplica = replica.get;
write();
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
  value.set('sibling');
}
consume(primary, replica);
copiedPrimary('snapshot');
copiedReplica('snapshot');
unrelated.get('unrelated');
