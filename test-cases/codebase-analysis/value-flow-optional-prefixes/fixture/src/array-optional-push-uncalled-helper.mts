import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
const spare = { client: { get: replacement } };
const args = [spare];
const box = { client: replica };
function append() { args.push(box); }
function mutate(_first, second = spare) { second.client.get = replacement; }
// The helper is intentionally never called: the suffix is definitely the runtime receiver.
mutate(...args, { client: untouched });
replica.get('uncalled helper leaves this receiver untouched');
untouched.get('fixed suffix receives the second parameter');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
