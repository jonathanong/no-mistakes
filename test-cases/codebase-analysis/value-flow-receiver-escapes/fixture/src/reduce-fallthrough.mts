import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
function mutate(acc, item) {
  acc.client.get = replacement;
  return acc;
}
declare const arr: any[];
declare const cb: any;
box = { client: replica };
// `box` is never passed. Unknown elements, a callback with no slot, and an
// unresolved callback must not invent a write.
arr.reduce(mutate);
[other, other].reduce(() => 0);
[other, other].reduce(cb);
replica.get('changed');
unrelated.get('control');
