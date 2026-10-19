import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(a, x) {
  x.client.get = replacement;
}
box = { client: replica };
// A leading hole is not an element. `box` lands on the second parameter.
mutate(...[, box]);
replica.get('changed');
unrelated.get('control');
