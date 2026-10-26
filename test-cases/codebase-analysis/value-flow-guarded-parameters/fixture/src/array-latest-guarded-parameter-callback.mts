import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
const other = { client: untouched };
function run(box) {
  let args = [box];
  box = other;
  if (flag) { args = [box]; }
  mutate(...args);
}
// Callback forwarding retains its existing conservative union, not an exact invocation snapshot.
[{ client: replica }].forEach(run);
replica.get('earlier callback argument');
untouched.get('later captured origin');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
