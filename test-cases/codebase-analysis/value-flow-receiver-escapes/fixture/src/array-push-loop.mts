import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
box = { client: replica };
function mutate(first, second) {
  first.client.get = replacement;
  second.client.get = replacement;
}
const args = [other];
// The pushed origin is live on the guarded path; the first origin always survives.
while (flag) { args.push(box); break; }
mutate(...args);
replica.get('earlier array origin');
untouched.get('later array origin');
primary.get('previous box client stays known');
replica.info('sibling member stays known');
unrelated.get('control');
