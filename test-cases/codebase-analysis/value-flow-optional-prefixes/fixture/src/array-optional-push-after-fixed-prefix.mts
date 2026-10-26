import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
function mutate(_first, second = spare) { second.client.get = replacement; }
const args = [];
if (flag) { args.push({ client: untouched }); }
// The fixed spread occupies slot 1; this ordinary argument must stay slot 2.
mutate(...[spare], { client: replica }, ...args);
replica.get('fixed ordinary argument is always written');
untouched.get('later optional tail cannot shift the fixed receiver');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
