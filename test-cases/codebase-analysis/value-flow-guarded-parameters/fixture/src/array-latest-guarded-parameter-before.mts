import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run(box, other) {
  box = other;
  let args = [box];
  if (flag) { args = [box]; }
  mutate(...args);
}
run({ client: replica }, { client: untouched });
replica.get('initial parameter origin');
untouched.get('replacement or shadow origin');
primary.get('unrelated or nested initial origin');
replica.info('sibling control');
unrelated.get('control');
