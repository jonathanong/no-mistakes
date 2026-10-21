import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
function mutate(value) { value.client.get = replacement; }
let args = [box];
if (flag) { args = [other]; }
// A separate direct call still owns definite invalidation of this receiver.
mutate(other);
mutate(...args);
replica.get('guarded earlier branch');
untouched.get('direct writer remains unknown');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
