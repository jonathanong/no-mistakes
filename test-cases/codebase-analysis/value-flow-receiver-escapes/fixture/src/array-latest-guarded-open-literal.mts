import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
declare const prefix: unknown[];
const definite = { client: primary };
function mutate(value) { value.client.get = replacement; }
let args = [box];
if (flag) { args = [other]; }
// Literal suffix recursion must preserve the same guarded/open uncertainty.
mutate(...prefix, ...args, ...[definite]);
replica.get('earlier shifted origin');
untouched.get('latest shifted origin');
primary.get('open prefix can contain this receiver or leave it untouched');
primary.info('sibling control');
unrelated.get('control');
