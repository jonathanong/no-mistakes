import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
const prefix = [];
const definite = { client: primary };
function mutate(value) { value.client.get = replacement; }
let args = [box];
if (flag) { args = [other]; }
// A known empty prefix leaves the guaranteed array slot before the suffix.
mutate(...prefix, ...args, definite);
replica.get('earlier shifted origin');
untouched.get('latest shifted origin');
primary.get('trailing suffix never reaches the first parameter');
primary.info('sibling control');
unrelated.get('control');
