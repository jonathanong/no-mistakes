import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
function mutate(a, b, x) {
  x.client.get = replacement;
}
box = { client: replica };
// `box` is parameter 2. The spread must not leave it on parameter 1.
mutate(...[other, other], box);
replica.get('changed');
unrelated.get('control');
primary.get('old');
