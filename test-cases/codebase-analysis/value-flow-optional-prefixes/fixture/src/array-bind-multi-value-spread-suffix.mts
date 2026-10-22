import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
const spare = { client: { get: replacement } };
const box = { client: replica };
const tail = { client: untouched };
function mutate(_first, second, _third) { second.client.get = replacement; }
// The first spread supplies thisArg and parameter zero; following bound
// arguments must retain their actual parameter positions.
const run = mutate.bind(...[null, spare], box, tail);
run();
replica.get('parameter one is replaced');
untouched.get('ordinary suffix remains the third parameter');
primary.get('primary receiver control');
replica.info('sibling member control');
unrelated.get('unrelated receiver control');
