import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
function mutate(value) { value.client.get = replacement; }
let box = { client: replica };
let args = [box];
// The same lexical name captures different objects on the two array paths.
box = { client: untouched };
if (flag) { args = [box]; }
mutate(...args);
replica.get('earlier captured object');
untouched.get('latest captured object');
primary.get('unrelated origin');
unrelated.get('control');
