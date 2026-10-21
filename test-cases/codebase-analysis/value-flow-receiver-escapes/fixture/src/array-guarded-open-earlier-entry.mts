import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
box = { client: replica };
function mutate(value) { value.client.get = replacement; }
let first = [];
let second = [box];
if (flag) { first = [other]; }
if (flag) { second = [other]; }
// With the first spread empty, the second spread's earlier entry occupies slot zero.
mutate(...first, ...second);
replica.get('earlier second-spread origin shifts left');
untouched.get('later guarded origins');
primary.get('previous box client stays known');
replica.info('sibling member stays known');
unrelated.get('control');
