import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(accumulator) {
  accumulator.client.get = replacement;
  return accumulator;
}
box = { client: replica };
[untouched].reduce(mutate, box);
// One element and no initial value never invokes the callback, so this adds
// no accumulator origin. Two or more elements are `reduce-no-initial.mts`.
[box].reduce(mutate);
[box].reduce(() => box, box);
replica.get('changed');
primary.get('old');
untouched.get('element');
