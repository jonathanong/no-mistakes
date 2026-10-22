import { primary, replica, untouched, unrelated } from '@vendor/valkey'; // guarded-array-receiver-api.test.js locks the call lines below.
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run(box, other) {
  let args = [box];
  if (flag) { args = [box]; }
  box = other;
  mutate(...args);
}
run({ client: replica }, { client: untouched });
replica.get('initial parameter origin');
untouched.get('replacement or shadow origin');
primary.get('unrelated or nested initial origin');
replica.info('sibling control');
unrelated.get('control');
