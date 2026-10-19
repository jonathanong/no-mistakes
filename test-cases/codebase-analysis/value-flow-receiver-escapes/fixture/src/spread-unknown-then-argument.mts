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
// `rest` has no known elements, so every later slot sees `box`.
mutate(...rest, box);
replica.get('changed');
unrelated.get('control');
