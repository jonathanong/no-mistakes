import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
const spare = { client: { get: replacement } };
function mutate(first, second = spare) { first.client.get = replacement; second.client.get = replacement; }
let args = [];
if (flag) { args = [box]; }
args.push(other);
// Runtime correlates both flag reads; the analysis retains a conservative union.
if (flag) { args = [other]; }
mutate(...args);
replica.get('earlier optional origin');
untouched.get('guarded origin is not a proven common slot');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
