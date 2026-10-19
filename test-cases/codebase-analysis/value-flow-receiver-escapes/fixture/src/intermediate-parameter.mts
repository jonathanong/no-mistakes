import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
function inner(x) {
  x.client.get = replacement;
}
function middle(b) {
  inner(b);
  // `b` is an intermediate parameter origin of `x`. Dropping it leaves this call Known.
  b.client.get('through-parameter');
}
let box = { client: primary };
box = { client: replica };
middle(box);
replica.get('changed');
primary.get('old');
unrelated.get('control');
