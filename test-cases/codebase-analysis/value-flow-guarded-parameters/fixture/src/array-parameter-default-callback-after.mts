import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
const other = { client: untouched };
function write(value) { value.client.get = replacement; }
function mutate(box) {
  const args = [box];
  box = other;
  write(...args);
}
function invoke(value, callback = mutate) { callback(value); }
invoke({ client: replica });
replica.get('default callback nested writer receives the captured argument');
untouched.get('reassignment happens after the captured argument');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
