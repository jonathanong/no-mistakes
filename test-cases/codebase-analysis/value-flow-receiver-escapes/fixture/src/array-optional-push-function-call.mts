import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
function mutate(_first, second = spare) { second.client.get = replacement; }
const args = [spare];
if (flag) { args.push({ client: replica }); }
// Function.prototype.call removes its receiver argument before the spread.
mutate.call(null, ...args, { client: untouched });
replica.get('call spread can occupy the second slot');
untouched.get('call suffix can occupy the second slot');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
