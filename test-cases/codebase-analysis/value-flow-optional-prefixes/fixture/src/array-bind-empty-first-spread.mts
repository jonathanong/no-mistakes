import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: untouched };
function mutate(first) { first.client.get = replacement; }
// An empty spread contributes no thisArg; the next ordinary argument is thisArg,
// and the following object is bound parameter zero.
const run = mutate.bind(...[], null, box);
run();
untouched.get('bind spread replaces this method');
replica.get('replica receiver control');
primary.get('primary receiver control');
replica.info('sibling member control');
unrelated.get('unrelated receiver control');
