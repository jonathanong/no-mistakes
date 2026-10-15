import { primary, other, replica, reset, copied, afterUnknown } from '@vendor/valkey';
declare function makeClient(): typeof primary;
// Each entry owns a fresh receiver; a previous entry cannot supply its tail.
function fresh() {
  const local = makeClient();
  local.get('key');
  local.get = replica.get;
}
fresh(); fresh();
// The same helper with distinct supplied objects must not share a tail fact.
function distinct(client: typeof primary) {
  client.get('key');
  client.get = replica.get;
}
distinct(primary); distinct(other);
// A reset at each loop entry overwrites the previous iteration's tail.
let current = reset;
for (const item of ['a', 'b']) {
  current = makeClient();
  current.get(item);
  current.get = replica.get;
}
// The RHS method is copied at its assignment, before the later receiver write.
for (const item of ['a', 'b']) {
  copied.get(item);
  copied.get = replica.get;
  replica.get = other.get;
}
declare const unknownMethod: (...args: unknown[]) => unknown;
// An overwritten unknown value must not survive the loop backedge.
for (const item of ['a', 'b']) {
  afterUnknown.get(item);
  afterUnknown.get = unknownMethod;
  afterUnknown.get = other.get;
}
