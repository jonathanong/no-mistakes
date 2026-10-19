import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
let box = { client: primary };
const other = { client: untouched };
function mutate(a, x) {
  x.client.get = replacement;
}
box = { client: replica };
// The nested spread opens the tail. `box` is a later visible element, so it is included.
mutate(...[...[other], box]);
replica.get('changed');
unrelated.get('control');
