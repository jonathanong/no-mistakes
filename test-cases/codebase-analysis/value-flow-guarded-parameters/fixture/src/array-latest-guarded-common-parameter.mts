import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run(box) {
  let args = [box];
  // The parameter binding is unchanged between the two array captures.
  if (flag) { args = [box]; }
  mutate(...args);
}
run({ client: replica });
replica.get('same parameter always reaches the writer');
replica.info('sibling control');
primary.get('unrelated origin');
untouched.get('unrelated origin');
unrelated.get('control');
