import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run() {
  let box = { client: primary };
  const other = { client: untouched };
  box = { client: replica };
  let args = [box];
  if (flag) { args = [other]; }
  // The forwarded writer must retain the possible channel across foreign replay.
  mutate(...args);
  replica.get('earlier origin stays possible');
  untouched.get('later origin remains historical unknown');
  primary.get('previous box client stays known');
  replica.info('sibling member stays known');
  unrelated.get('control');
}
run();
