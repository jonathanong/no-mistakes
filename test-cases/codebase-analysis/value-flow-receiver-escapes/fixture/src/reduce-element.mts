import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(acc, item) {
  item.client.get = replacement;
  return acc;
}
box = { client: replica };
[box].reduce(mutate, 0);
replica.get('changed');
unrelated.get('control');
