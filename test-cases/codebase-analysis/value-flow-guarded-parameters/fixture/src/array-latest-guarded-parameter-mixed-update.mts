import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { if (value && value.client) { value.client.get = replacement; } }
function run(box, other) {
  box++;
  // The update has no assignment-RHS receipt; do not claim an initial-parameter snapshot.
  let args = [box];
  box = other;
  if (flag) { args = [box]; }
  mutate(...args);
}
run({ client: replica }, { client: untouched });
replica.get('unmatched update retains the historical approximation');
untouched.get('later guarded argument');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
