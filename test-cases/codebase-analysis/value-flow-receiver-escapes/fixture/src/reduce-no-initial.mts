import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const other = { client: untouched };
function mutate(acc, item) {
  acc.client.get = replacement;
  return acc;
}
box = { client: replica };
[box, other].reduce(mutate);
replica.get('changed');
unrelated.get('control');
