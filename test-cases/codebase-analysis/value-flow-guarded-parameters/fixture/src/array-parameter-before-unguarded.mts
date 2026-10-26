import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run(box, other) {
  // Removing the guard must not hide the same capture-time recall gap.
  box = other;
  const args = [box];
  mutate(...args);
}
run({ client: replica }, { client: untouched });
replica.get('original argument was never passed to mutate');
untouched.get('captured replacement was mutated');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
