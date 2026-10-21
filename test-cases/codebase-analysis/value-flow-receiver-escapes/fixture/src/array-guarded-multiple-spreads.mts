import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
box = { client: replica };
function mutate(value) { value.client.get = replacement; }
let first = [];
let second = [];
if (flag) { first = [other]; }
if (flag) { second = [other]; }
const tail = [box];
// Either guarded spread may be empty, allowing the const tail into slot zero.
mutate(...first, ...second, ...tail, ...[]);
replica.get('const tail may supply the first parameter');
untouched.get('guarded arrays may supply the first parameter');
primary.get('previous box client stays known');
replica.info('sibling member stays known');
unrelated.get('control');
