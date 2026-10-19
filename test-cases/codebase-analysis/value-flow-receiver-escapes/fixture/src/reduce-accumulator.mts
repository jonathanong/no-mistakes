import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(accumulator) {
  accumulator.client.get = replacement;
  return accumulator;
}
box = { client: replica };
[untouched].reduce(mutate, box);
// No initial value, and a callback with no bare slot, add no accumulator origin.
[box].reduce(mutate);
[box].reduce(() => box, box);
replica.get('changed');
primary.get('old');
untouched.get('element');
