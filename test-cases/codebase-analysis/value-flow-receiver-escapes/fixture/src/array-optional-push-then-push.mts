import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
const box = { client: replica };
const other = { client: untouched };
function mutate(_first, second = spare) { second.client.get = replacement; }
let args = [spare];
if (flag) { args.push(box); }
// This unconditional push occupies slot one without the guard and slot two with it.
args.push(other);
mutate(...args);
replica.get('optional push can be the second argument');
untouched.get('later push can shift into the second argument');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
