import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: replica };
function mutate(first) { first.client.get = replacement; }
// bind's first array spread supplies thisArg and bound parameter zero.
const run = mutate.bind(...[null, box]);
run();
replica.get('bind spread replaces this method');
untouched.get('untouched receiver control');
primary.get('primary receiver control');
replica.info('sibling member control');
unrelated.get('unrelated receiver control');
