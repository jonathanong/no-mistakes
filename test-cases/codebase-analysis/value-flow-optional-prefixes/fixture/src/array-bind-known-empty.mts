import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
const args = [];
function mutate(first) { first.client.get = replacement; }
// A proven empty prefix leaves the bound suffix in the first formal exactly.
const run = mutate.bind(null, ...args, { client: untouched });
run();
replica.get('unrelated positive control');
untouched.get('the local replacement always runs');
primary.get('unrelated positive control');
replica.info('sibling member control');
unrelated.get('unrelated control');
