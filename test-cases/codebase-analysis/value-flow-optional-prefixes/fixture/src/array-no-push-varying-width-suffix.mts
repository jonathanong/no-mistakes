import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
let args = [];
if (flag) { args = [spare]; }
function mutate(_first, second = spare) { second.client.get = replacement; }
mutate(...args, { client: untouched });
replica.get('unrelated positive control');
untouched.get('varying prefix may leave this suffix untouched or write it');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
