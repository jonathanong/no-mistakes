import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) {
  value.client.get = replacement;
}
box = { client: replica };
[box].filter(mutate);
[box].some(mutate);
[box].every(mutate);
[box].find(mutate);
[box].findIndex(mutate);
[box].findLast(mutate);
[box].findLastIndex(mutate);
[box].flatMap(mutate);
replica.get('changed');
unrelated.get('control');
