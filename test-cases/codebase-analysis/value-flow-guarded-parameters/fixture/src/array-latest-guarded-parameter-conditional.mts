import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run(box, other) {
  let args = [box];
  if (flag) { box = other; }
  if (flag) { args = [box]; }
  mutate(...args);
}
run({ client: replica }, { client: untouched });
replica.get('initial parameter candidate');
untouched.get('other parameter candidate or unrelated control');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
