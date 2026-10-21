import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
const box = { client: untouched };
function outer() {
  function run() {
    let args = [box];
    if (flag) { args = [box]; }
    mutate(...args);
  }
  // Local predeclaration must select this binding rather than the outer box.
  const box = { client: replica };
  run();
}
outer();
replica.get('same local captured object is always passed');
replica.info('sibling remains precise');
primary.get('unrelated origin');
untouched.get('shadowed outer object remains precise');
unrelated.get('control');
