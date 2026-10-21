import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
const tail = { client: primary };
function mutate(first, second) {
  first.client.get = replacement;
  second.client.get = replacement;
}
let args = [box];
if (flag) { args = [other]; }
// The assignment is guarded; this later push is a guaranteed argument.
args.push(tail);
mutate(...args);
replica.get('earlier branch');
untouched.get('latest branch');
primary.get('guaranteed pushed argument');
primary.info('sibling control');
unrelated.get('control');
