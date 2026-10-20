import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
function mutate(value) { value.client.get = replacement; }
box = { client: replica };
// The spread runs while args is still [box]. The later array is not an element.
let args = [box];
mutate(...args);
args = [other];
replica.get('changed');
untouched.get('later');
unrelated.get('control');
