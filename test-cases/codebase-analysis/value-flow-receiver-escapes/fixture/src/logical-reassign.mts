import { primary, replica, untouched, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function mutate(value) {
  // `value` may stay nullish or become `box`. The RHS is a possible receiver.
  value ??= box;
  value.client.get = replacement;
}
box = { client: replica };
mutate(null);
replica.get('changed');
unrelated.get('control');
primary.get('old');
untouched.get('also');
