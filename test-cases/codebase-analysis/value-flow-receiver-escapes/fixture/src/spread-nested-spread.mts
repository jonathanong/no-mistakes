import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
let box = { client: primary };
const other = { client: untouched };
function mutate(a, x) {
  x.client.get = replacement;
}
box = { client: replica };
// The nested spread occupies slot 0. `box` is the next element, so it is parameter 1.
mutate(...[...[other], box]);
replica.get('changed');
unrelated.get('control');
