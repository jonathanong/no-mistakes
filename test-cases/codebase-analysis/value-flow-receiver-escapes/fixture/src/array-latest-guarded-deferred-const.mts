import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
function run() {
  let args = [box];
  if (flag) { args = [box]; }
  mutate(...args);
}
// Deferred execution sees the const although its declaration follows the body.
const box = { client: replica };
run();
replica.get('same captured object is always passed');
replica.info('sibling remains precise');
primary.get('unrelated origin');
untouched.get('unrelated origin');
unrelated.get('control');
