import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run(box) {
  let args = [box];
  // The update rebinds the parameter, rather than mutating its object members.
  box++;
  if (flag) { args = [box]; }
  mutate(...args);
}
run({ client: replica });
replica.get('initial parameter candidate');
untouched.get('other parameter candidate or unrelated control');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
