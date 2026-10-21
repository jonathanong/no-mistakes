import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
const box = { client: replica };
const args = [];
if (flag) { args.push(box); }
function mutate(first = spare) { first.client.get = replacement; }
// Either receiver can occupy the first formal after the bound thisArg is removed.
const run = mutate.bind(null, ...args, { client: untouched });
run();
replica.get('optional push may supply the first formal');
untouched.get('skipped push moves the suffix into the first formal');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
