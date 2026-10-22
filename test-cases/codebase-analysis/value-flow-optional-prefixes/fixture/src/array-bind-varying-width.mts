import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
let args = [];
if (flag) { args = [spare]; }
function mutate(first = spare) { first.client.get = replacement; }
// bind removes thisArg before projecting the uncertain spread and suffix.
const run = mutate.bind(null, ...args, { client: untouched });
run();
replica.get('unrelated positive control');
untouched.get('empty prefix writes this receiver; a nonempty prefix does not');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
