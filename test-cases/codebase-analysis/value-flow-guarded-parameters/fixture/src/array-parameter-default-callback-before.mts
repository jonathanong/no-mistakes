import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
const other = { client: untouched };
function write(value) { value.client.get = replacement; }
function mutate(box) {
  box = other;
  const args = [box];
  write(...args);
}
function invoke(value, callback = mutate) { callback(value); }
invoke({ client: replica });
replica.get('default callback reassigns before capturing the argument');
untouched.get('nested writer receives the reassigned parameter');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
