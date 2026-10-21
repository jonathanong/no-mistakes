import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
const args = [spare];
const box = { client: replica };
function append() { args.push(box); }
function mutate(_first, second = spare) { second.client.get = replacement; }
if (flag) { append(); }
mutate(...args, { client: untouched });
replica.get('conditional helper may place this receiver in the second slot');
untouched.get('conditional push may shift this suffix into the second slot');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
