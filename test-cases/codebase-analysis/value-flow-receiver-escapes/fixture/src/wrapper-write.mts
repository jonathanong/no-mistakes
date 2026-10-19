import { primary, replica, untouched } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// The write is one wrapper deeper than the object argument.
function inner(value) { value.client.get = replacement; }
function mutate(box) { inner(box); }
box = { client: replica };
mutate(box);
// A parameter passed to itself is a cycle; the outer argument is still a receiver.
function again(value) {
  again(value);
  value.client.get = replacement;
}
again(box);
replica.get('changed');
primary.get('old');
untouched.get('control');
