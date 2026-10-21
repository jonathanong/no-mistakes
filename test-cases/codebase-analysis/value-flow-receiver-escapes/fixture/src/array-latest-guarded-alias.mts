import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
function mutate(value) { value.client.get = replacement; }
const run = mutate;
let args = [box];
if (flag) { args = [other]; }
run(...args);
replica.get('earlier branch');
untouched.get('latest branch through alias');
primary.get('unrelated origin');
replica.info('sibling control');
unrelated.get('control');
