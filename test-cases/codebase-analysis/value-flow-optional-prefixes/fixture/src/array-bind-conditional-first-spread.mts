import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
const box = { client: replica };
const args = [null];
if (flag) { args.push(box); }
function mutate(first = { client: untouched }) { first.client.get = replacement; }
// The optional bound parameter shifts the ordinary suffix between zero and
// one; either imported receiver is replaced on a reachable runtime path.
const run = mutate.bind(...args, { client: untouched });
run();
replica.get('optional bound parameter may replace this method');
untouched.get('shifted suffix may replace this method');
primary.get('primary receiver control');
replica.info('sibling member control');
unrelated.get('unrelated receiver control');
