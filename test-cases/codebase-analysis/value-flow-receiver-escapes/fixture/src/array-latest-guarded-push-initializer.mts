import { primary, replica, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const spare = { client: { get: replacement } };
function mutate(first, second = spare) { second.client.get = replacement; }
let args = [];
// This push changes the previous array; the replacement holds its numeric result.
args = [args.push(box)];
if (flag) { args = [box, box]; }
mutate(...args);
replica.get('both guarded slots remain possible');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
