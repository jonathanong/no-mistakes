import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
box = { client: replica };
function mutate(first, second) {
  first.client.get = replacement;
  second.client.get = replacement;
}
let args = [box];
// Slot 0 is the same value on both paths; slot 1 exists only on the guarded path.
if (flag) { args = [box, other]; }
mutate(...args);
replica.get('shared first slot');
untouched.get('guarded second slot');
primary.get('previous box client stays known');
unrelated.get('control');
