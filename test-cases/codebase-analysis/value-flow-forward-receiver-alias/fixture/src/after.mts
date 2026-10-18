import { primary, replica, unrelated } from '@vendor/forward-after';
function replacement() {}
let box = { client: primary };
function rebind() { box = { client: replica }; }
const before = box;
rebind();
const after = box;
const copiedPrimary = primary.get;
const copiedReplica = replica.get;
function write() {
  before.client.get = replacement;
  after.client.get = replacement;
}
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
