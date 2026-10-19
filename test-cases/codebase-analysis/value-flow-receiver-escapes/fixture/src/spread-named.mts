import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(x) {
  x.client.get = replacement;
}
box = { client: replica };
const args = [box];
mutate(...args);
replica.get('changed');
unrelated.get('control');
