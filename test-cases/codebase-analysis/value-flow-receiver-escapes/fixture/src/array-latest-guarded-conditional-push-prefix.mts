import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
const spare = { client: { get: replacement } };
function mutate(first, second = spare) { first.client.get = replacement; second.client.get = replacement; }
let args = [box];
if (flag) { args.push(other); }
// The conditional push cannot prove the second slot exists on every path.
if (flag) { args = [box, other]; }
mutate(...args);
replica.get('guaranteed common prefix');
untouched.get('optional pushed tail');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
