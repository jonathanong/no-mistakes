import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
function mutate(value) { value.client.get = replacement; }
box = { client: replica };
// The live array at the spread is [other], so replica is not an element.
let args = [box];
args = [other];
mutate(...args);
replica.get('changed');
untouched.get('written');
unrelated.get('control');
