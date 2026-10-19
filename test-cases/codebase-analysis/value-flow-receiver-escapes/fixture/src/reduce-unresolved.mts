import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function list() {
  return [box, box];
}
function mutate(acc) {
  acc.client.get = replacement;
}
box = { client: replica };
// No initial value, and the receiver is a call rather than an array.
// `box` is not recorded as the accumulator.
list().reduce(mutate);
replica.get('changed');
primary.get('old');
unrelated.get('control');
