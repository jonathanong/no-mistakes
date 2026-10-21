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
// An already-open spread must keep the guarded baseline separate from this suffix.
mutate(...prefix, ...args, definite);
replica.get('earlier shifted origin');
untouched.get('latest shifted origin');
primary.get('ordinary open suffix remains definite');
primary.info('sibling control');
unrelated.get('control');
