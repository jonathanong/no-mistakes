import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) {
  value ||= box;
  value.client.get = replacement;
}
box = { client: replica };
mutate(null);
replica.get('changed');
primary.get('old');
unrelated.get('control');
