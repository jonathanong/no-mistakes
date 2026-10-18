import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const copied = primary.get;
const copiedReplica = replica.get;
// The root is acquired by write, while the nested closure owns the mutation.
function write() { const b = box; return () => { b.client.get = replacement; }; }
function rebind() { box = { client: replica }; }
rebind();
write()();
function consume(value, changed, other, before, after) {
  value.info('sibling');
  changed.info('sibling');
  other.get('unrelated');
  before('copied-primary');
  after('copied-replica');
}
consume(primary, replica, unrelated, copied, copiedReplica);
