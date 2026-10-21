import { primary, replica, untouched, unrelated } from '@vendor/valkey'; // guarded-array-receiver-api.test.js locks lines 12–16.
declare const flag: boolean;
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
box = { client: replica };
function mutate(value) { value.client.get = replacement; }
let args = [box];
// The earlier array remains live when this guarded assignment does not run.
if (flag) { args = [other]; }
mutate(...args);
replica.get('earlier array origin');
untouched.get('later array origin');
primary.get('previous box client stays known');
replica.info('sibling member stays known');
unrelated.get('control');
