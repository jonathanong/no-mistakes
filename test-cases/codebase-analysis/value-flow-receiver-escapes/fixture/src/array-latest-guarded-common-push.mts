import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
function mutate(value) { value.client.get = replacement; }
let args = [];
args.push(box);
// Both paths pass box at slot zero, including the path established by push.
if (flag) { args = [box]; }
mutate(...args);
replica.get('always mutated common slot');
replica.info('sibling control');
primary.get('unrelated origin');
untouched.get('unrelated origin');
unrelated.get('control');
