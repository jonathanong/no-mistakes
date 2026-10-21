import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
const spare = { client: { get: replacement } };
function mutate(first, second = spare) { first.client.get = replacement; second.client.get = replacement; }
const late = { client: primary };
let args = [box];
args.push(other);
// Only the prefix is common; the two possible tails have distinct identities.
if (flag) { args = [box, late]; }
mutate(...args);
replica.get('guaranteed common prefix');
untouched.get('earlier pushed tail');
primary.get('latest guarded tail');
replica.info('sibling control');
unrelated.get('control');
