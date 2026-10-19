import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
function mutate(x) {
  x.client.get = replacement;
}
box = { client: replica };
// Neither nested element is `box`, so the direct replica call stays known.
mutate(...[...[other], other]);
replica.get('changed');
unrelated.get('control');
