import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function list() {
  return [box];
}
function mutate(kept, x) {
  x.client.get = replacement;
}
box = { client: replica };
// `list()` is neither an array nor a name, so the spread stays open.
// A later element and a later argument still reach `x`. `kept` is before that tail.
mutate(unrelated, ...list(), ...[box]);
mutate(unrelated, ...list(), box);
replica.get('changed');
primary.get('old');
unrelated.get('control');
