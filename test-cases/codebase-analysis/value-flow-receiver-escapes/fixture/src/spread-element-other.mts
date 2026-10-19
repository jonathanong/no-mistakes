import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
let box = { client: primary };
const other = { client: untouched };
function mutate(x) {
  x.client.get = replacement;
}
box = { client: replica };
// Neither arm is `box`, so the direct `replica.get` call stays known.
mutate(...[flag ? other : other]);
replica.get('changed');
unrelated.get('control');
