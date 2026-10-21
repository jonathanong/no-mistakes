import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run(box, other) {
  // The array retains its object even after the parameter binding changes.
  const args = [box];
  box = other;
  mutate(...args);
}
run({ client: replica }, { client: untouched });
replica.get('captured original argument was mutated');
untouched.get('replacement argument was never passed to mutate');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
