import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate([acc], item) {
  void acc;
  item.client.get = replacement;
}
box = { client: replica };
// The accumulator pattern has no bare slot. The element parameter still sees `box`.
[box, box].reduce(mutate);
replica.get('changed');
primary.get('old');
unrelated.get('control');
