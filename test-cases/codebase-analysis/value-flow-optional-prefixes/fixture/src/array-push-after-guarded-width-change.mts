import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
const box = { client: replica };
let args = [];
if (flag) { args = [spare]; }
// The unconditional push lands at slot 0 or slot 1 depending on the guarded width.
args.push(box);
function mutate(_first, second = spare) { second.client.get = replacement; }
mutate(...args, { client: untouched });
replica.get('the appended box may occupy the second formal');
untouched.get('the suffix may shift into the second formal');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
