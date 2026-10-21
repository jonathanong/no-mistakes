import { replica, tail, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const late = { client: tail };
const other = { client: untouched };
function mutate(value) { value.client.get = replacement; }
let args = [];
if (flag) { args = [other]; }
mutate(...args, box);
// The same consumer's next call still needs its new trailing argument origin.
mutate(...args, late);
replica.get('first trailing origin');
tail.get('second trailing origin');
untouched.get('historical latest origin');
replica.info('sibling control');
unrelated.get('control');
