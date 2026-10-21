import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run(box, other) {
  let args = [box];
  let local;
  // Only box is a parameter; its destructuring default supplies the second capture.
  [box = other, local] = [];
  if (flag) { args = [box]; }
  mutate(...args);
}
run({ client: replica }, { client: untouched });
replica.get('earlier captured object');
untouched.get('later default origin');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
