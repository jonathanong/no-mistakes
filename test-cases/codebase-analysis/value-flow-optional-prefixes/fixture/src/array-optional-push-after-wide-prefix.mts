import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
function mutate(_first, second = spare) { second.client.get = replacement; }
const args = [];
if (flag) { args.push({ client: untouched }); }
// Both fixed prefix elements precede replica; the optional tail cannot move it into slot 2.
mutate(...[spare, spare], { client: replica }, ...args);
replica.get('replica remains after the written second formal');
untouched.get('optional tail remains after the written second formal');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
