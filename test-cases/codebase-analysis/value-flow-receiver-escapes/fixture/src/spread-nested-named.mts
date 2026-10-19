import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(x) {
  x.client.get = replacement;
}
box = { client: replica };
const named = [box];
// A nested spread of a named array uses that array's elements.
mutate(...[...named]);
replica.get('changed');
unrelated.get('control');
