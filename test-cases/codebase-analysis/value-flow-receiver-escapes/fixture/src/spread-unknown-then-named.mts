import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
let box = { client: primary };
const other = { client: untouched };
function mutate(a, b, x) {
  x.client.get = replacement;
}
declare const rest: any[];
box = { client: replica };
const named = [box];
// Named arrays stay on the entry path. An earlier unresolved spread keeps them open.
mutate(...rest, ...named);
replica.get('changed');
unrelated.get('control');
