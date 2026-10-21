import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
function mutate(_first, second = spare) { second.client.get = replacement; }
const args = [];
if (flag) { args.push({ client: untouched }); }
// Function.prototype.call removes its receiver; the fixed spread and box still precede the optional tail.
mutate.call(null, ...[spare], { client: replica }, ...args);
replica.get('fixed ordinary argument is written locally, not called');
untouched.get('later optional tail cannot shift the fixed receiver');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
